use crate::{BitSet};
use parking_lot::Mutex;
use std::{
    any::Any,
    slice::SliceIndex,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle, cell::Cell,
};

use crate::{SliceTuple, ThreadPoolScope};

// Shared arc that represents a pointer tuple
type BoxedPtrTuple = Arc<dyn Any + Send + Sync + 'static>;

// Data passed to each thread
pub(super) struct ThreadFuncEntry {
    pub(super) base: BoxedPtrTuple,
    pub(super) batch_length: usize,
    pub(super) batch_offset: usize,
}

// Shared arc that represents a shared function
pub(super) type BoxedFunction =
    Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'static>;

// Represents a single task that will be executed by multiple threads
pub(super) enum ThreadedTask {
    // This is a single task that will be executed on a single thread
    Execute(Box<dyn FnOnce() + Send>),

    // Executed in multiple threads
    ForEachBatch {
        entry: ThreadFuncEntry,
        function: BoxedFunction,
    },
}

// Keeps track of the index of the thread we are currently running on
thread_local! {
    static CURRENT: Cell<usize> = Cell::new(0);
}

// A single threadpool that contains multiple worker threads that are ready to be executed in parallel
// TODO: Maybe move this to another crate, like a "utils" crate?
pub struct ThreadPool {
    // Task sender and receiver
    task_sender: Option<Sender<ThreadedTask>>,
    task_receiver: Arc<Mutex<Receiver<ThreadedTask>>>,

    // Number of tasks that are waiting to get executed
    waiting: Arc<AtomicU32>,

    // Number of active threads currently working
    active: Arc<AtomicU32>,

    // Join handles for the OS threads
    joins: Vec<JoinHandle<()>>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::with(Self::default_thread_count())
    }
}

impl ThreadPool {
    // Calculate the default thread count
    pub fn default_thread_count() -> usize {
        num_cpus::get() * 8
    }

    // Create a new thread pool with a specific number of threads
    pub fn with(num: usize) -> Self {
        let (task_sender, task_receiver) =
            std::sync::mpsc::channel::<ThreadedTask>();

        // Create a simple threadpool
        let mut threadpool = Self {
            active: Arc::new(AtomicU32::new(0)),
            joins: Default::default(),
            waiting: Arc::new(AtomicU32::new(0)),
            task_sender: Some(task_sender),
            task_receiver: Arc::new(Mutex::new(task_receiver)),
        };

        // Spawn the worker threads
        let joins = (0..num)
            .into_iter()
            .map(|i| spawn(&threadpool, i))
            .collect::<Vec<_>>();
        threadpool.joins = joins;

        threadpool
    }

    // Given an immutable/mutable slice of elements, run a function over all of them elements in parallel
    // If specified, this will use a bitset to hop over useless entries
    // Warning: This will not wait till all the threads have finished executing their specific functions
    pub(crate) fn for_each_async<'a, I: for<'i> SliceTuple<'i>>(
        &'a mut self,
        mut list: I,
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + 'a,
        bitset: Option<BitSet>,
        batch_size: usize,
    ) {
        // If the slices have different lengths, we must abort
        let length = list.slice_tuple_len();
        assert!(
            length.is_some(),
            "Cannot have slice with different lengths"
        );
        let length = length.unwrap();

        // Create the scheduler config
        let batch_size = batch_size.max(1);
        let num_tasks =
            (length as f32 / batch_size as f32).ceil() as usize;
        let remaining = length % batch_size;

        // Internal function that is either used in a single thread or in multiple threads
        let internal =
            move |ptrs: &mut I,
                  length: usize,
                  offset: usize,
                  bitset: Option<&BitSet>| {
                if let Some(bitset) = &bitset {
                    // With a bitset filter
                    let mut i = 0;
                    while i < length {
                        // Check the next entry that is valid (that passed the filter)
                        if let Some(hop) =
                            bitset.find_one_from(i + offset)
                        {
                            i = hop;
                        } else {
                            return;
                        }

                        function(unsafe {
                            I::get_unchecked(ptrs, i)
                        });
                        i += 1;
                    }
                } else {
                    // Without a bitset filter
                    for i in 0..length {
                        function(unsafe {
                            I::get_unchecked(ptrs, i)
                        });
                    }
                }
            };

        // Run the code in a single thread if needed
        if num_tasks == 1 {
            internal(&mut list, length, 0, bitset.as_ref());
            return;
        }

        // Box the function into an arc
        type ArcFn<'b> =
            Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'b>;

        // The bitset is going to be a shareable bitset instead
        let bitset = bitset.map(Arc::new);

        let function: ArcFn<'a> =
            Arc::new(move |entry: ThreadFuncEntry| unsafe {
                // Optionally, the user might specify a specific bitset
                let bitset = bitset.clone();

                // Decompose the thread entry into it's raw components
                let offset = entry.batch_offset;
                let ptrs = entry.base.downcast::<I::PtrTuple>().ok();
                let length = entry.batch_length;
                let mut slices = ptrs
                    .map(|ptrs| {
                        I::from_ptrs(
                            &ptrs,
                            entry.batch_length,
                            offset,
                        )
                    })
                    .unwrap();

                // Call the internal function
                internal(
                    &mut slices,
                    length,
                    offset,
                    bitset.as_deref(),
                );
            });

        // Convert the lifetimed arc into a static arc
        let function: ArcFn<'static> = unsafe {
            std::mem::transmute::<ArcFn<'a>, ArcFn<'static>>(function)
        };

        // Run the function in mutliple threads
        let base: Arc<dyn Any + Send + Sync> =
            Arc::new(list.as_ptrs());
        for batch_index in 0..num_tasks {
            self.append(ThreadedTask::ForEachBatch {
                entry: ThreadFuncEntry {
                    base: base.clone(),
                    batch_length: if batch_index == (num_tasks - 1)
                        && remaining > 0
                    {
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
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + 'a,
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
        function: impl Fn(<I as SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + 'a,
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
    pub fn execute<F: FnOnce() + Send + 'static>(
        &mut self,
        function: F,
    ) {
        let task = ThreadedTask::Execute(Box::new(function));
        self.append(task);
    }

    // Create a scope that we can use to send multiple commands to the threads
    pub fn scope<'a>(
        &'a mut self,
        function: impl FnOnce(&mut ThreadPoolScope<'a>),
    ) {
        let mut scope = ThreadPoolScope { pool: self };

        function(&mut scope);
        drop(scope);
    }

    // Get the number of threads that are stored in the thread pool
    pub fn num_threads(&self) -> usize {
        self.joins.len()
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
        while self.num_active_threads() > 0
            || self.num_idling_jobs() > 0
        {
            std::hint::spin_loop();
        }
    }

    // Get the index of the current thread
    pub fn current() -> usize {
        CURRENT.with(|current| current.get())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
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
    let name = format!("WorkerThread-{index}");

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            // Set the thread index at the start
            CURRENT.with(|current| current.set(index+1));

            loop {
                // No task, block so we shall wait
                let task =
                    if let Ok(task) = task_receiver.lock().recv() {
                        task
                    } else {
                        break;
                    };

                // The thread woke up, so we must fetch the highest priority task now
                active.fetch_add(1, Ordering::Relaxed);
                match task {
                    // Execute a single task in another thread
                    ThreadedTask::Execute(f) => f(),

                    // Execute the same function over and over again on the same slice, but at different indices (no overrun)
                    ThreadedTask::ForEachBatch {
                        entry,
                        function,
                    } => function(entry),
                }

                // Update the active thread atomic and waiting task atomic
                waiting.fetch_sub(1, Ordering::Relaxed);
                active.fetch_sub(1, Ordering::Relaxed);
            }
        })
        .unwrap()
}
