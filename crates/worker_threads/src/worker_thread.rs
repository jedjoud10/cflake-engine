use std::{marker::PhantomData, sync::{atomic::{AtomicPtr, AtomicUsize, AtomicBool, Ordering}, Barrier, Arc, RwLock}};
use crate::SharedData;
// We must shutdown the threads
pub(crate) static SHUTDOWN: AtomicBool = AtomicBool::new(false);

// Create a new thread
pub fn new<C: 'static, T: Sync + 'static>(thread_index: usize, barriers: Arc<(Barrier, Barrier, Barrier)>, shared_data: Arc<RwLock<SharedData<C, T>>>) {
    std::thread::spawn(move || {
        // Wait until the barrier allows us to continue
        let (barrier, end_barrier, shutdown_barrier) = barriers.as_ref();
        let ptr = shared_data.as_ref();
        loop {
            barrier.wait();
            // Execute the code that was given, if valid
            let ptr = ptr.read().unwrap();
            let idx = thread_index;
                let data = unsafe { &*ptr };
                let context = unsafe { &*data.context };
                let elements = unsafe { &*data.elements };
                
                // Calculate the indices
                let start_idx = idx * data.chunk_size;
                let end_idx = (idx+1) * data.chunk_size;
                for i in start_idx..end_idx {
                    // Execute the function
                    let elem = elements.get(i);
                    if let Some(&elem) = elem {
                        // Unsafe magic
                        let elem = unsafe { &mut *elem };
                        (data.function)(context, elem);
                    }
                }
                
            // Check if we must shutdown
            if SHUTDOWN.load(Ordering::Relaxed) {
                shutdown_barrier.wait();
                break;
            }
            end_barrier.wait();
        }
    });
}