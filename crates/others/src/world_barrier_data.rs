use std::sync::{
    atomic::{AtomicBool, Ordering},
    Barrier, Condvar, RwLock,
};

// Internal
pub struct WorldBarrierDataInternal {
    pub end_frame_sync_barrier: Barrier,
    pub quit_loop_sync_barrier: Barrier,
    pub world_valid: AtomicBool,
    pub world_destroyed: AtomicBool,
}

// Some world barrier data type shit used for syncing the child threads
pub struct WorldBarrierData {
    internal: RwLock<Option<WorldBarrierDataInternal>>,
}

impl WorldBarrierData {
    // New uninitialized
    pub fn new_uninit() -> Self {
        Self { internal: RwLock::new(None) }
    }
    // New (Though we don't make a new RwLock)
    pub fn new_update(&self, n: usize) {
        let mut writer_ = self.internal.write().unwrap();
        let writer = &mut *writer_;
        *writer = Some(WorldBarrierDataInternal {
            end_frame_sync_barrier: Barrier::new(n),
            quit_loop_sync_barrier: Barrier::new(n),
            world_valid: AtomicBool::new(false),
            world_destroyed: AtomicBool::new(false),
        });
    }

    // We are destroying the world
    pub fn destroying_world(&self) {
        let r = &self.internal.read().unwrap();
        let world_destroyed = &r.as_ref().unwrap().world_destroyed;
        world_destroyed.store(true, Ordering::Relaxed);
    }
    // The world has finalized it's initialization
    pub fn init_finished_world(&self) {
        let r = &self.internal.read().unwrap();
        let world_valid = &r.as_ref().unwrap().world_valid;
        world_valid.store(true, Ordering::Relaxed);
    }
    // Is the world initialized
    pub fn is_world_valid(&self) -> bool {
        let r = &self.internal.read().unwrap();
        match r.as_ref() {
            Some(x) => {
                let world_valid = &x.world_valid;
                world_valid.load(Ordering::Relaxed)
            }
            None => false,
        }
    }
    // Is the world destroyed
    pub fn is_world_destroyed(&self) -> bool {
        let r = &self.internal.read().unwrap();
        match r.as_ref() {
            Some(x) => {
                let world_destroyed = &x.world_destroyed;
                world_destroyed.load(Ordering::Relaxed)
            }
            None => false,
        }
    }
    // We have finished the frame for this specific thread, so wait until all the threads synchronise
    pub fn thread_sync(&self) {
        let r = &self.internal.read().unwrap();
        let end_frame_sync_barrier = &r.as_ref().unwrap().end_frame_sync_barrier;
        //println!("Called ThreadSync on thread {:?}", std::thread::current().id());
        let result = (end_frame_sync_barrier).wait();
    }
    // Sync up the quit barrier
    pub fn thread_sync_quit(&self) {
        let r = &self.internal.read().unwrap();
        let end_frame_sync_barrier = &r.as_ref().unwrap().end_frame_sync_barrier;
        //println!("Called ThreadSyncQuit on thread {:?}", std::thread::current().id());
        let result = (end_frame_sync_barrier).wait();
    }
}
