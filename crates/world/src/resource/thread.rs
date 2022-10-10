use std::{thread::{ThreadId, JoinHandle}, slice::SliceIndex, ffi::c_void, sync::{Barrier, atomic::{AtomicU32, Ordering, AtomicBool}, Arc}, os::windows::thread};
use parking_lot::{RwLock, Condvar, Mutex};

// Untyped raw pointer
type UntypedPtr = usize;
type UntypedMutPtr = usize;

// Represents a single task that will be executed by multiple threads
pub enum ThreadedTask {
    Execute(Box<dyn FnOnce() + Send + Sync>),
    ForEachBatch {
        base: UntypedPtr,
        total_length: usize,
        batch_size: usize,
        function: Box<dyn FnOnce(UntypedPtr) + Send + Sync>
    },
    ForEachMutBatch {
        base: UntypedMutPtr,
        total_length: usize,
        batch_size: usize,
        function: Box<dyn FnOnce(UntypedMutPtr) + Send + Sync>
    },
}

// Shared task pool
type TaskPool = RwLock<Vec<(ThreadedTask, i32)>>;

// A single threadpool that contains multiple worker threads that are ready to be executed in parallel
pub struct ThreadPool {
    tasks: Arc<TaskPool>,
    shutdown: Arc<AtomicBool>,
    active: Arc<AtomicU32>,
    condvar: Arc<(Mutex<bool>, Condvar)>,
    joins: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    // Create a new thread pool with the default number of threads
    pub fn new() -> Self {
        // Create a simple threadpool
        let num = num_cpus::get();
        let mut threadpool = Self {
            tasks: Default::default(),
            shutdown: Arc::new(AtomicBool::new(false)),
            active: Arc::new(AtomicU32::new(num as u32)),
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
            joins: Default::default(),
        };
        
        // Spawn the worker threads
        let joins = (0..num).into_iter().map(|i| spawn(&threadpool, i)).collect::<Vec<_>>();
        threadpool.joins = joins;
        
        threadpool        
    }

    // Given an immutable slice of elements, run a function over all of them elements in parallel
    pub fn for_each<T: Sync>(&mut self, list: &[T], function: impl FnOnce(&T) + Send) {}

    // Given a mutable slice of elements, run a function over all of them elemeents in parallel
    pub fn for_each_mut<T: Sync>(&mut self, list: &mut [T], function: impl FnOnce(&mut T) + Send) {}

    // Add a new task to execute in the threadpool. The task will have default priority
    // The returned handle allows us to check when the task has finished executing
    pub fn execute(&mut self, function: impl FnOnce() + Send + Sync + 'static) {
        self.execute_with_priority(function, 0);
    }

    // Add a new task to execute in the threadpool with a specific priority
    // The returned handle allows us to check when the task has finished executing
    pub fn execute_with_priority(&mut self, function: impl FnOnce() + Send + Sync + 'static, priority: i32) {
        let task = ThreadedTask::Execute(Box::new(function));
        self.tasks.write().push((task, priority));
        self.condvar.1.notify_one();
        *self.condvar.0.lock() = true;
    }

    // Get the number of threads that are stored in the thread pool
    pub fn num_threads(&self) -> usize {
        self.joins.len()
    }

    // Jobs waiting to get ran by other threads
    pub fn num_idling_jobs(&self) -> usize {
        self.tasks.read().len()
    }
    
    // Get the number of threads that are currently working
    pub fn num_active_threads(&self) -> usize {
        self.active.load(Ordering::Relaxed) as usize
    }

    // Sort the threadpool's tasks based on their priority
    pub fn sort(&mut self) {
        
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        while self.num_active_threads() != 0 || self.num_idling_jobs() != 0 {}
        self.shutdown.store(true, Ordering::Relaxed);
        *self.condvar.0.lock() = true;
        self.condvar.1.notify_all();
        for x in self.joins.drain(..) {
            x.join().unwrap();
        }
    }
}

// Initialize a worker thread from a threadpool and start it's task pulling loop
fn spawn(threadpool: &ThreadPool, index: usize) -> JoinHandle<()> {
    let tasks = threadpool.tasks.clone();
    let shutdown = threadpool.shutdown.clone();
    let active = threadpool.active.clone();
    let condvar = threadpool.condvar.clone();
    let name = format!("WorkerThread-{index}");

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            loop {
                // No task, block on the condvar to no waste CPU cycles
                if tasks.read().is_empty() {
                    let (mutex, condvar) = &*condvar;
                    active.fetch_sub(1, Ordering::Relaxed);
                    let mut lock = mutex.lock();
                    condvar.wait(&mut lock);
                    *lock = false;

                    // Break from the loop if necessary
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    // Wait till the task comes
                    while tasks.read().is_empty() {}

                    active.fetch_add(1, Ordering::Relaxed);
                }

                // The thread woke up, so we must fetch the highest priority task now
                if let Some((task, _)) = tasks.write().pop() {
                    match task +{
                        ThreadedTask::Execute(f) => f(),
                        ThreadedTask::ForEachBatch { base, total_length, batch_size, function } => todo!(),
                        ThreadedTask::ForEachMutBatch { base, total_length, batch_size, function } => todo!(),
                    }
                }
            }
    }).unwrap()
}
