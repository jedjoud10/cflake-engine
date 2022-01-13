use std::sync::{Barrier, atomic::{AtomicPtr, Ordering::Relaxed}, Arc, RwLock};

use crate::{SharedData, SHUTDOWN};

// A thread pool that contains multiple WorkerThreadInitData, 
// so we can send messages to the threads to tell them to execute something, and we will wait until all of them have executed
pub struct ThreadPool<C, T: Sync> {
    // A barrier that we can use to sync up the threads for execution
    // The second barrier is used after every execution, juuust to make sure
    // The third barrier is the shutdown barrier, so we all the threads shut down in sync
    barriers: Arc<(Barrier, Barrier, Barrier)>,
    // Also store it's pointer, since we need to update it
    arc: Arc<RwLock<SharedData<C, T>>>,
    // The numbers of threads that we have in total
    max_thread_count: usize,
} 

impl<C: 'static, T: Sync + 'static> ThreadPool<C, T> {
    // Create a new thread pool
    pub fn new(max_thread_count: usize) -> Self {
        // Barrier stuff
        let barriers = Arc::new((Barrier::new(max_thread_count+1), Barrier::new(max_thread_count+1), Barrier::new(max_thread_count+1)));
        // Data
        let arc = Arc::new(RwLock::new(SharedData::<C, T>::default()));
        // Create the threads
        for i in 0..max_thread_count {
            crate::worker_thread::new(i, barriers.clone(), arc.clone());
        }
        
        Self {
            max_thread_count,
            barriers,
            arc,
        }
    }
    // Divide the task between the multiple threads, and invoke them
    pub fn execute(&self, elements: &mut Vec<T>, context: &C, task: fn(&C, usize, &mut T)) {
        let (barrier, end_barrier, shutdown_barrier) = self.barriers.as_ref();
        {
            // Update the value, then unlock
            let mut shared_data = self.arc.write().unwrap();
            // Convert the &mut Vec<T> to Vec<*mut T>
            shared_data.elements = elements.into_iter().map(|x| x as *mut T).collect::<Vec<_>>();
            shared_data.function = task;
            shared_data.context = context as *const C;
            // Calculate the chunk size
            const MIN_CHUNK_SIZE: usize = 64;
            shared_data.chunk_size = ((elements.len() / self.max_thread_count) + 1).max(MIN_CHUNK_SIZE);
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