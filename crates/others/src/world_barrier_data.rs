use lazy_static::lazy_static;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Barrier, Condvar, RwLock,
};

lazy_static! {
    static ref BARRIERS_WORLD: Arc<WorldBarrierData> = Arc::new(WorldBarrierData::new_uninit());
}
// Initialize the world barrier data with the specified amount of threads to wait for
pub fn init(n: usize) {
    let x = BARRIERS_WORLD.as_ref();
    x.new_update(n);
}
// As ref
pub fn as_ref() -> &'static WorldBarrierData {
    BARRIERS_WORLD.as_ref()
}

// Internal
pub struct WorldBarrierDataInternal {
    // Frame syncing
    pub end_frame_sync_barrier: Barrier,
    // Quitting
    pub quit_loop_sync_barrier: Barrier,
    // Atomics
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
            // We don't sync the render loop
            end_frame_sync_barrier: Barrier::new(n-1),
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
        (end_frame_sync_barrier).wait();
    }
    // Sync up the quit barrier
    pub fn thread_sync_quit(&self) {
        let r = &self.internal.read().unwrap();
        let quit_loop_sync_barrier = &r.as_ref().unwrap().quit_loop_sync_barrier;
        (quit_loop_sync_barrier).wait();
    }
}
