use std::sync::{
    atomic::{AtomicPtr, Ordering::Relaxed},
    Arc, Barrier, RwLock,
};

use crate::{SharedData, SHUTDOWN};

// A thread pool that contains multiple WorkerThreadInitData,
// so we can send messages to the threads to tell them to execute something, and we will wait until all of them have executed
pub struct ThreadPool<T> {
    // A barrier that we can use to sync up the threads for execution
    // The second barrier is used after every execution, juuust to make sure
    // The third barrier is the shutdown barrier, so we all the threads shut down in sync
    barriers: Arc<(Barrier, Barrier, Barrier)>,
    // Also store it's pointer, since we need to update it
    arc: Arc<RwLock<SharedData<T>>>,
    // The numbers of threads that we have in total
    max_thread_count: usize,
}

impl<T: 'static> ThreadPool<T> {
    // Create a new thread pool
    pub fn new<F: Fn() + 'static + Sync + Send>(max_thread_count: usize, init_function: F) -> Self {
        // Barrier stuff
        let barriers = Arc::new((Barrier::new(max_thread_count + 1), Barrier::new(max_thread_count + 1), Barrier::new(max_thread_count + 1)));
        // Data
        let arc = Arc::new(RwLock::new(SharedData::<T>::default()));
        let init_function_arc = Arc::new(Box::new(init_function));
        // Create the threads
        for i in 0..max_thread_count {
            crate::worker_thread::new(i, init_function_arc.clone(), barriers.clone(), arc.clone());
        }

        Self { max_thread_count, barriers, arc }
    }
    // Get the maximum numbers of threads that are in the pool
    pub fn max_thread_count(&self) -> usize {
        self.max_thread_count
    }
    // Divide the task between the multiple threads, and invoke them
    pub fn execute<F: Fn(&mut T) + 'static + Sync + Send>(&self, elements: &mut Vec<T>, task: F) {
        let (barrier, end_barrier, shutdown_barrier) = self.barriers.as_ref();
        {
            // Update the value, then unlock
            let mut shared_data = self.arc.write().unwrap();
            // Convert the &mut Vec<T> to Vec<*mut T>
            let length = elements.len();
            shared_data.elements = elements.iter_mut().map(|x| x as *mut T).collect::<Vec<_>>();
            shared_data.function = Box::new(task);
            // Calculate the chunk size
            shared_data.chunk_size = (length / self.max_thread_count) + 1;
            // Now we can unlock
        }
        barrier.wait();

        // The threads and running their functions...

        // We wait until all of them finished
        end_barrier.wait();
    }
    // Shutdown
    pub fn shutdown(&self) {
        SHUTDOWN.store(true, Relaxed);
        self.barriers.as_ref().2.wait();
    }
}
