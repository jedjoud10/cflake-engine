use std::{thread::{ThreadId, JoinHandle}, slice::SliceIndex, ffi::c_void, sync::{Barrier, atomic::{AtomicU32, Ordering, AtomicBool}, Arc, mpsc::{Sender, Receiver}}, os::windows::thread, mem::size_of, any::Any, marker::PhantomData};
use parking_lot::{RwLock, Condvar, Mutex};

// Untyped raw pointer
type UntypedPtr = usize;
type UntypedMutPtr = usize;

// Represents a single task that will be executed by multiple threads
pub enum ThreadedTask {
    // This is a single task that will be executed on a single thread
    Execute(Box<dyn FnOnce() + Send>),

    ForEachBatch {
        base: UntypedPtr,
        batch_length: usize,
        batch_offset: usize,
        size_of: usize,
        function: Arc<dyn Fn(UntypedPtr) + Send + Sync>
    },
    ForEachMutBatch {
        base: UntypedMutPtr,
        batch_length: usize,
        batch_offset: usize,
        size_of: usize,
        function: Arc<dyn Fn(UntypedMutPtr) + Send + Sync>
    },
}

/*
// Results given from the execute tasks
type Result = Box<dyn Any + Send>;

// Task handle allows us to fetch the results of specific tasks
pub struct AsyncTaskHandle<T: Send + 'static> {
    id: u64,
    _phantom: PhantomData<T>,
}
*/

// A single threadpool that contains multiple worker threads that are ready to be executed in parallel
pub struct ThreadPool {
    sender: Option<Sender<ThreadedTask>>,
    receiver: Arc<Mutex<Receiver<ThreadedTask>>>,
    waiting: Arc<AtomicU32>,
    active: Arc<AtomicU32>,
    joins: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    // Create a new thread pool with the default number of threads
    pub fn new() -> Self {
        // Le sender and le receiver
        let (sender, receiver) = std::sync::mpsc::channel::<ThreadedTask>();

        // Create a simple threadpool
        let num = num_cpus::get() * 4;
        let mut threadpool = Self {
            active: Arc::new(AtomicU32::new(0)),
            joins: Default::default(),
            waiting: Arc::new(AtomicU32::new(0)),
            sender: Some(sender),
            receiver: Arc::new(Mutex::new(receiver)),
        };
        
        // Spawn the worker threads
        let joins = (0..num).into_iter().map(|i| spawn(&threadpool, i)).collect::<Vec<_>>();
        threadpool.joins = joins;
        
        threadpool        
    }

    /*
    // Given an immutable slice of elements, run a function over all of them elements in parallel
    pub fn for_each<T: Sync + Send + 'static>(&mut self, list: &[T], function: impl Fn(&T) + Send + Sync + 'static) {
        let batch_size = list.len() / self.num_threads();
        let remaining = list.len() % self.num_threads();
        let task: Arc<dyn Fn(UntypedPtr) + Send + Sync> = Arc::new(move |ptr: UntypedPtr| unsafe { 
            let ptr = ptr as *const T;
            function(&*ptr);
        });

        for index in 0..self.num_threads() {
            let offset = batch_size * index;
            let length = batch_size + ((index == self.num_threads() - 1) as usize * remaining);

            let task = ThreadedTask::ForEachBatch { base: list.as_ptr() as usize, batch_length: length, batch_offset: offset, function: task.clone(), size_of: size_of::<T>() };
            self.append(task);
        }
        
        self.join();
    }

    // Given a mutable slice of elements, run a function over all of them elemeents in parallel
    pub fn for_each_mut<T: Sync + Send + 'static>(&mut self, list: &mut [T], function: impl Fn(&mut T) + Send + Sync + 'static) {}
    */

    // Execute a raw task. Only should be used internally
    pub fn append(&mut self, task: ThreadedTask) {
        self.waiting.fetch_add(1, Ordering::Relaxed);
        self.sender.as_ref().unwrap().send(task).unwrap();
    }

    // Add a new task to execute in the threadpool. This task will run in the background
    pub fn execute(&mut self, function: impl FnOnce() + Send + 'static) {
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
        while self.num_active_threads() > 0 || self.num_idling_jobs() > 0 {}
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take().unwrap();
        self.join();

        for x in self.joins.drain(..) {
            x.join().unwrap();
        }
    }
}

// Initialize a worker thread from a threadpool and start it's task pulling loop
fn spawn(threadpool: &ThreadPool, index: usize) -> JoinHandle<()> {
    let receiver = threadpool.receiver.clone();
    let active = threadpool.active.clone();
    let waiting = threadpool.waiting.clone();
    let name = format!("WorkerThread-{index}");

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            loop {
                // No task, block so we shall wait
                let task = if let Ok(task) = receiver.lock().recv() {
                    task
                } else {
                    break;
                };
                
                // The thread woke up, so we must fetch the highest priority task now
                active.fetch_add(1, Ordering::Relaxed);
                match task {
                    ThreadedTask::Execute(f) => f(),
                    ThreadedTask::ForEachBatch { base, batch_length, batch_offset, size_of, function } => {
                        let start = base + batch_offset * size_of;
                        let end = start + batch_length * size_of;
                        
                        for i in (start..end).step_by(size_of) {
                            function(i);
                        }
                    },
                    ThreadedTask::ForEachMutBatch { base, batch_length, batch_offset, size_of, function } => todo!(),
                }                
                waiting.fetch_sub(1, Ordering::Relaxed);
                active.fetch_sub(1, Ordering::Relaxed);
            }
    }).unwrap()
}
