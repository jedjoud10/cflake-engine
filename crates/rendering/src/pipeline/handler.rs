use std::sync::{atomic::AtomicBool, Arc, Barrier, Mutex};
// Data that will be sent back to the main thread after we start the pipeline thread
pub struct PipelineHandler {
    // The thread handle for the render thread, so we can join it to the main thread at any time
    pub handle: std::thread::JoinHandle<()>,
    // A barrier that we can use to sync up with the main thread at the start of each frame
    pub sbarrier: Arc<Barrier>,
    // A barrier that we can use to sync up with the main thread at the end of each frame
    pub ebarrier: Arc<Barrier>,
    // An atomic we use to shutdown the render thread
    pub eatomic: Arc<AtomicBool>,
    // An atomic telling us if we are waiting for the sbarrier to start the frame
    pub waiting: Arc<AtomicBool>,
    // Some timing data that we will share with the pipeline
    pub time: Arc<Mutex<(f64, f64)>>,
}
