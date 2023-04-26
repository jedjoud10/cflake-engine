use super::{ArcFn, ThreadFuncEntry, ThreadedTask, ThreadedExecuteTaskResult};
use crate::{BitSet};
use crate::{SliceTuple, ThreadPoolScope};

use ahash::AHashMap;
use crossbeam_channel::{Sender, Receiver};
use parking_lot::Mutex;
use std::any::TypeId;
use std::{
    any::Any,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

// A single threadpool that contains multiple worker threads that are ready to be executed in parallel
pub struct ThreadPool {
    // Task sender and receiver
    pub(super) task_sender: Option<Sender<ThreadedTask>>,
    pub(super) task_receiver: Receiver<ThreadedTask>,

    // Task result sender and receiver
    pub(crate) task_results_sender: Sender<ThreadedExecuteTaskResult>,
    pub(crate) task_results_receiver: Receiver<ThreadedExecuteTaskResult>,

    // Number of tasks that are waiting to get executed
    pub(crate) waiting: Arc<AtomicU32>,

    // Number of active threads currently working
    pub(crate) active: Arc<AtomicU32>,
    pub(crate) panicked: Arc<Mutex<Option<usize>>>,

    // Join handles for the OS threads
    pub(crate) joins: Vec<JoinHandle<()>>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::with(num_cpus::get())
    }
}

impl ThreadPool {
    // Create a new thread pool with a specific number of threads
    pub fn with(num: usize) -> Self {
        let (task_sender, task_receiver) = crossbeam_channel::unbounded::<ThreadedTask>();
        let (task_results_sender, task_results_receiver) = crossbeam_channel::unbounded::<ThreadedExecuteTaskResult>();

        // Create a simple threadpool
        let mut threadpool = Self {
            active: Arc::new(AtomicU32::new(0)),
            joins: Default::default(),
            waiting: Arc::new(AtomicU32::new(0)),
            task_sender: Some(task_sender),
            task_receiver,
            panicked: Arc::new(Mutex::new(None)),
            task_results_sender,
            task_results_receiver,
        };

        // Spawn the worker threads
        let joins = (0..num).map(|i| spawn(&threadpool, i)).collect::<Vec<_>>();
        threadpool.joins = joins;
        log::debug!("Initialized a new thread pool with {num} thread(s)");

        threadpool
    }

    // Given an immutable/mutable slice of elements, run a function over all of them elements in parallel
    // If specified, this will use a bitset to hop over useless entries
    // Warning: This will not wait till all the threads have finished executing their specific functions
    pub(crate) fn for_each_async<'a, I: for<'i> SliceTuple<'i>>(
        &'a mut self,
        mut list: I,
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple) + Send + Sync + 'a,
        bitset: Option<BitSet>,
        batch_size: usize,
    ) {
        // If the slices have different lengths, we must abort
        let length = list.slice_tuple_len();
        assert!(length.is_some(), "Cannot have slice with different lengths");
        let length = length.unwrap();

        // Create the scheduler config
        let batch_size = batch_size.max(1);
        let num_tasks = (length as f32 / batch_size as f32).ceil() as usize;
        let remaining = length % batch_size;
        log::trace!("for_each_async: elems: {length}, batch size: {batch_size}, threads: {num_tasks}, remaining: {remaining}");

        // Internal function that will be wrapped within a closure and executed on the main thread / other threads
        // This will simply loop over all the elements specified by 'ptrs', 'length', and 'offset'
        fn iterate<I: for<'i> SliceTuple<'i>, F: Fn(<I as SliceTuple>::ItemTuple) + Send + Sync>(
            ptrs: &mut I,
            length: usize,
            offset: usize,
            function: &F,
            bitset: Option<&BitSet>,
        ) {
            if let Some(bitset) = &bitset {
                // With a bitset filter
                let mut i = 0;
                while i < length {
                    // Check the next entry that is valid (that passed the filter)
                    if let Some(hop) = bitset.find_one_from(i + offset) {
                        i = hop - offset;
                    } else {
                        return;
                    }

                    function(unsafe { I::get_unchecked(ptrs, i) });
                    i += 1;
                }
            } else {
                // Without a bitset filter
                for i in 0..length {
                    function(unsafe { I::get_unchecked(ptrs, i) });
                }
            }
        }

        // Run the code in a single thread if needed
        if num_tasks == 1 {
            iterate(&mut list, length, 0, &function, bitset.as_ref());
            return;
        }

        // The bitset is going to be a shareable bitset instead
        let bitset = bitset.map(Arc::new);

        let function: ArcFn<'a> = Arc::new(move |entry: ThreadFuncEntry| {
            // Optionally, the user might specify a specific bitset
            let bitset = bitset.clone();

            // Decompose the thread entry into it's raw components
            let offset = entry.batch_offset;
            let ptrs = entry.base.downcast::<I::PtrTuple>().ok();
            let length = entry.batch_length;
            let mut slices = ptrs
                .map(|ptrs| unsafe { I::from_ptrs(&ptrs, entry.batch_length, offset) })
                .unwrap();

            // Call the internal function
            iterate(&mut slices, length, offset, &function, bitset.as_deref());
        });

        // Convert the lifetimed arc into a static arc
        let function: ArcFn<'static> =
            unsafe { std::mem::transmute::<ArcFn<'a>, ArcFn<'static>>(function) };

        // Run the function in mutliple threads
        let base: Arc<dyn Any + Send + Sync> = Arc::new(list.as_ptrs());
        for batch_index in 0..num_tasks {
            self.append(ThreadedTask::ForEachBatch {
                entry: ThreadFuncEntry {
                    base: base.clone(),
                    batch_length: if batch_index == (num_tasks - 1) && remaining > 0 {
                        remaining
                    } else {
                        batch_size
                    },
                    batch_offset: batch_index * batch_size,
                },
                function: function.clone(),
            });
        }
    }

    // Given an immutable/mutable slice of elements, run a function over all of them elements in parallel
    // This function will wait untill all of the threads have finished executing their tasks
    pub fn for_each<'a, I: for<'i> SliceTuple<'i>>(
        &'a mut self,
        list: I,
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple) + Send + Sync + 'a,
        batch_size: usize,
    ) {
        self.for_each_async(list, function, None, batch_size);
        self.join();
    }

    // Given an immutable/mutable slice of elements, run a function over certain elements in parallel
    // This function will wait untill all of the threads have finished executing their tasks
    pub fn for_each_filtered<'a, I: for<'i> SliceTuple<'i>>(
        &'a mut self,
        list: I,
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple) + Send + Sync + 'a,
        bitset: BitSet,
        batch_size: usize,
    ) {
        self.for_each_async(list, function, Some(bitset), batch_size);
        self.join();
    }

    // Execute a raw task. Only should be used internally
    pub(super) fn append(&mut self, task: ThreadedTask) {
        self.waiting.fetch_add(1, Ordering::Relaxed);
        self.task_sender.as_ref().unwrap().send(task).unwrap();
    }

    // Add a new task to execute in the threadpool. This task will run in the background
    pub fn execute<R: Send + 'static, F: FnOnce() -> R + Send + 'static>(&mut self, function: F) {
        let task = ThreadedTask::Execute(Box::new(move || Box::new(function())), self.task_results_sender.clone());
        self.append(task);
    }

    // Create a scope that we can use to send multiple commands to the threads
    pub fn scope<'a>(&'a mut self, function: impl FnOnce(&mut ThreadPoolScope<'a>)) {
        let mut scope = ThreadPoolScope { pool: self };
        function(&mut scope);
        scope.pool.join();
        drop(scope);
    }

    // Get the number of threads that are stored in the thread pool
    pub fn num_threads(&self) -> usize {
        self.joins.len()
    }

    // Check if any of the threads have panicked, and return the thread ID
    pub(crate) fn check_any_panicked(&self) -> Option<usize> {
        *self.panicked.lock()
    }

    // Get the number of jobs that are waiting to get executed
    pub fn num_idling_jobs(&self) -> usize {
        self.waiting.load(Ordering::Relaxed) as usize
    }

    // Get the number of threads that are currently executing
    pub fn num_active_threads(&self) -> usize {
        self.active.load(Ordering::Relaxed) as usize
    }

    // Wait till all the threads finished executing
    pub fn join(&self) {
        while self.num_active_threads() > 0 || self.num_idling_jobs() > 0 {
            std::hint::spin_loop();
        }
    }

    // Fetch the results 
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.check_any_panicked().is_some() {
            return;
        }

        self.task_sender.take().unwrap();
        self.join();

        for x in self.joins.drain(..) {
            x.join().unwrap();
        }
    }
}

// Initialize a worker thread from a threadpool and start it's task pulling loop
fn spawn(threadpool: &ThreadPool, index: usize) -> JoinHandle<()> {
    let task_receiver = threadpool.task_receiver.clone();
    let active = threadpool.active.clone();
    let waiting = threadpool.waiting.clone();
    let name = format!("worker-{index}");

    // Used to check if a thread has panicked
    struct Hook(usize, Arc<Mutex<Option<usize>>>);
    impl Drop for Hook {
        fn drop(&mut self) {
            if std::thread::panicking() {
                let mut locked = self.1.lock();
                *locked = Some(self.0);
            }
        }
    }
    let hook = Hook(index, threadpool.panicked.clone());

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            let _hook = hook;

            loop {
                // No task, block so we shall wait
                // TODO: Please change this to a better type scheduler this is shit
                // TODO: Pwease optimize
                let task = if let Ok(task) = task_receiver.recv() {
                    task
                } else {
                    break;
                };

                // The thread woke up, so we must fetch the highest priority task now
                active.fetch_add(1, Ordering::Relaxed);
                super::execute(task);

                // Update the active thread atomic and waiting task atomic
                waiting.fetch_sub(1, Ordering::Relaxed);
                active.fetch_sub(1, Ordering::Relaxed);
            }
        })
        .unwrap()
}
