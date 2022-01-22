use crate::SharedData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Barrier, RwLock,
};
// We must shutdown the threads
pub(crate) static SHUTDOWN: AtomicBool = AtomicBool::new(false);

// Create a new thread
pub fn new<F: Fn() + 'static + Sync + Send, T: 'static>(
    thread_index: usize,
    init_function_arc: Arc<Box<F>>,
    barriers: Arc<(Barrier, Barrier, Barrier)>,
    shared_data: Arc<RwLock<SharedData<T>>>,
) {
    std::thread::spawn(move || {
        let init_function = (&*init_function_arc).as_ref();
        init_function();
        let (barrier, end_barrier, shutdown_barrier) = barriers.as_ref();
        let ptr = shared_data.as_ref();
        loop {
            // Wait until the barrier allows us to continue
            barrier.wait();
            //println!("Executing thread '{}'", thread_index);
            // Execute the code that was given, if valid
            let ptr = ptr.read().unwrap();
            let idx = thread_index;
            let data = &*ptr;
            let function = data.function.as_ref().unwrap();
            let function = unsafe { &**function };
            let elements = &*data.elements;

            // Calculate the indices
            let start_idx = idx * data.chunk_size;
            let end_idx = (idx + 1) * data.chunk_size;
            // If our start index is not in range, we can skip
            //dbg!(start_idx);
            //dbg!(end_idx);
            if start_idx < elements.len() {
                let mut count = 0;
                for i in start_idx..end_idx {
                    // Execute the function
                    let elem = elements.get(i);
                    if let Some(&elem) = elem {
                        // Unsafe magic
                        let elem = unsafe { &mut *elem };
                        function(elem);
                        count += 1;
                    }
                }
                //println!("Finished executing thread '{}', executed '{}'", thread_index, count);
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
