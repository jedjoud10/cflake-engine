use std::{thread::{ThreadId, JoinHandle}, slice::SliceIndex, ffi::c_void, sync::{Barrier, atomic::{AtomicU32, Ordering, AtomicBool}, Arc, mpsc::{Sender, Receiver}}, os::windows::thread, mem::size_of, any::Any, marker::PhantomData, num};
use parking_lot::{RwLock, Condvar, Mutex};
use crate::RefSliceTuple;

use super::{UntypedPtr, UntypedMutPtr};

// Shared arc that represents a pointer tuple
type BoxedPtrTuple = Arc<dyn Any + Send + Sync + 'static>;

// Certified moment
struct ThreadFuncEntry {
    base: BoxedPtrTuple,
    batch_length: usize,
    batch_offset: usize,
}

// Shared arc that represents a shared function
type BoxedFunction = Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'static>;

// Represents a single task that will be executed by multiple threads
enum ThreadedTask {
    // This is a single task that will be executed on a single thread
    Execute(Box<dyn FnOnce() + Send>),

    // Executed in multiple threads
    ForEachBatch {
        entry: ThreadFuncEntry,
        function: BoxedFunction,
    },

    /*
    // This is a singular batch of a bigger immutable tuple of slices
    ForEachBatchTuple {
        base: Arc<dyn TuplePtr>,
        batch_length: usize,
        batch_offset: usize,
        function: Arc<dyn Fn(&dyn TuplePtr) + Send + Sync>
    }

    // This is a singular batch of a bigger mutable tuple of slices
    ForEachBatchTupleMut {
        base: Arc<dyn TupleMutPtr>,
        batch_length: usize,
        batch_offset: usize,
        function: Arc<dyn Fn(&dyn TupleMutPtr) + Send + Sync>
    }
    */
}


// Task handle allows us to fetch the results of specific tasks
pub struct AsyncTaskHandle<T: Send + 'static> {
    id: u64,
    _phantom: PhantomData<T>,
}

// A single threadpool that contains multiple worker threads that are ready to be executed in parallel
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

impl ThreadPool {
    // Create a new thread pool with the default number of threads
    pub fn new() -> Self {
        let (task_sender, task_receiver) = std::sync::mpsc::channel::<ThreadedTask>();

        // Create a simple threadpool
        let num = num_cpus::get() * 8;
        let mut threadpool = Self {
            active: Arc::new(AtomicU32::new(0)),
            joins: Default::default(),
            waiting: Arc::new(AtomicU32::new(0)),
            task_sender: Some(task_sender),
            task_receiver: Arc::new(Mutex::new(task_receiver)),
        };
        
        // Spawn the worker threads
        let joins = (0..num).into_iter().map(|i| spawn(&threadpool, i)).collect::<Vec<_>>();
        threadpool.joins = joins;
        
        threadpool        
    }

    // Given an immutable slice of elements, run a function over all of them elements in parallel
    pub fn for_each<'s: 'i, 'i, I: RefSliceTuple<'s, 'i>>(&mut self, list: I, function: impl Fn(I::ItemRefTuple) + Send + Sync + 'static, batch_size: usize) {
        // If the slices have different lengths, we abort
        let len = list.slice_tuple_len();
        assert!(len.is_some(), "Cannot have slice with different lengths");
        let len = len.unwrap();

        // Make sure the inputs are valid and calculate the number of threads we will need
        let batch_length = batch_size.max(1);
        let num_threads_to_use = (len as f32 / batch_length as f32).ceil() as usize;
        let remaining = len % batch_length;
        
        // Run the code in a single thread if needed
        if num_threads_to_use == 1 {
            for x in 0..len {
                function(unsafe { list.get_unchecked(x) });
            }
            return;
        }

        // Box the function into an arc
        let function: BoxedFunction = Arc::new(move |entry: ThreadFuncEntry| unsafe {
            let offset = entry.batch_offset;
            let ptrs = I::from_boxed_ptrs(entry.base, entry.batch_length, offset).unwrap();

            for i in 0..entry.batch_length {
                let tuple = I::get_unchecked(&ptrs, i);
                function(tuple);
            }
        });
        
        // Run the function in mutliple threads
        let base = unsafe { I::to_boxed_ptrs(list) };
        for batch_index in 0..num_threads_to_use {
            self.append(ThreadedTask::ForEachBatch {
                entry: ThreadFuncEntry { 
                    base: base.clone(),
                    batch_length: if batch_index == (num_threads_to_use-1) && remaining != 0 {
                        remaining
                    } else { batch_size },
                    batch_offset: batch_index * batch_length,
                },
                function: function.clone(),
            });
        }
        
        self.join();
    }

    /*
    // Given a mutable slice of elements, run a function over all of them elemeents in parallel
    pub fn for_each_mut<T: Sync + Send + 'static>(&mut self, list: &mut [T], function: impl Fn(&mut T) + Send + Sync + 'static, batch_size: usize) {
        // Make sure the inputs are valid and calculate the number of threads we will need
        let batch_size = batch_size.max(1);
        let num_threads_to_use = (list.len() as f32 / batch_size as f32).ceil() as usize;
        let remaining = list.len() % batch_size;

        // Run the code in a single thread if needed
        if num_threads_to_use == 1 {
            list.iter_mut().for_each(function);
            return;
        }

        // Create the task as a clonable Arc since we will share it a lot
        let task: Arc<dyn Fn(UntypedMutPtr) + Send + Sync> = Arc::new(move |ptr: UntypedMutPtr| unsafe { 
            let ptr: *mut T = ptr.into();
            function(&mut *ptr);
        });

        // Iterate over the thread index, and append a task at a time
        for index in 0..num_threads_to_use {
            let is_last = index == num_threads_to_use - 1;
            let offset = batch_size * index;
            let length = if is_last && remaining != 0 {
                remaining
            } else { batch_size };

            // Send the task to the global queue
            let task = ThreadedTask::ForEachMutBatch { base: list.as_mut_ptr().into(), batch_length: length, batch_offset: offset, function: task.clone(), size_of: size_of::<T>() };
            self.append(task);
        }
        
        self.join();
    }
    */

    // Execute a raw task. Only should be used internally
    fn append(&mut self, task: ThreadedTask) {
        self.waiting.fetch_add(1, Ordering::Relaxed);
        self.task_sender.as_ref().unwrap().send(task).unwrap();
    }

    // Add a new task to execute in the threadpool. This task will run in the background
    pub fn execute<F: FnOnce() + Send + 'static>(&mut self, function: F) {
        let task = ThreadedTask::Execute(Box::new(function));
        self.append(task);
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
        while self.num_active_threads() > 0 || self.num_idling_jobs() > 0 {
            std::hint::spin_loop();
        }
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
            loop {
                // No task, block so we shall wait
                let task = if let Ok(task) = task_receiver.lock().recv() {
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
                    ThreadedTask::ForEachBatch { entry, function } => unsafe {
                        function(entry);
                    },

                    /*
                    ThreadedTask::ForEachMutBatch { base, batch_length, batch_offset, size_of, function } => unsafe {
                        let base = Into::<*mut ()>::into(base) as usize;
                        let start = base + batch_offset * size_of;
                        let end = base + batch_length * size_of;
                        
                        for i in (start..end).step_by(size_of) {
                            function(UntypedMutPtr::from(i as *mut ()));
                        }
                    },
                    */
                } 

                // Update the active thread atomic and waiting task atomic
                waiting.fetch_sub(1, Ordering::Relaxed);
                active.fetch_sub(1, Ordering::Relaxed);
            }
    }).unwrap()
}
