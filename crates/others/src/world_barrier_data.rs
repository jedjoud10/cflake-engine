use std::sync::{Barrier, atomic::{AtomicBool, Ordering}, Condvar};

// Some world barrier data type shit used for syncing the child threads
pub struct WorldBarrierData {
    pub end_frame_sync_barrier: Barrier,
    pub quit_loop_sync_barrier: Barrier,
    pub world_valid: AtomicBool,
    pub world_destroyed: AtomicBool,
}

impl WorldBarrierData {
    // New
    pub fn new(n: usize) -> Self {
        Self {
            end_frame_sync_barrier: Barrier::new(n),
            quit_loop_sync_barrier: Barrier::new(n),
            world_valid: AtomicBool::new(false),
            world_destroyed: AtomicBool::new(false),
        }
    }

    // We are destroying the world
    pub fn destroying_world(&self) {
        self.world_destroyed.store(true, Ordering::Relaxed);
    }
    // The world has finalized it's initialization
    pub fn init_finished_world(&self) {
        self.world_valid.store(true, Ordering::Relaxed);
    }
    // Is the world initialized
    pub fn is_world_valid(&self) -> bool {
        self.world_valid.load(Ordering::Relaxed)
    }
    // Is the world destroyed
    pub fn is_world_destroyed(&self) -> bool {
        self.world_destroyed.load(Ordering::Relaxed)
    }
    // We have finished the frame for this specific thread, so wait until all the threads synchronise
    pub fn thread_sync(&self) {
        //println!("Called ThreadSync on thread {:?}", std::thread::current().id());
        let result = (&self.end_frame_sync_barrier).wait();
    }
    // Sync up the quit barrier
    pub fn thread_sync_quit(&self) {
        //println!("Called ThreadSyncQuit on thread {:?}", std::thread::current().id());
        let result = (&self.quit_loop_sync_barrier).wait();
    }
}