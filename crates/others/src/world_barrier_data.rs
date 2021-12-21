use lazy_static::lazy_static;
use std::{sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Barrier, Condvar, RwLock,
}, collections::HashMap, thread::ThreadId};

lazy_static! {
    static ref BARRIERS_WORLD: Arc<WorldBarrierData> = Arc::new(WorldBarrierData::new_uninit());
}
// Initialize the world barrier data with the specified amount of threads to wait for
pub fn init(thread_ids: Vec<ThreadId>) {
    let x = BARRIERS_WORLD.as_ref();
    x.new_update(thread_ids);
}
// As ref
pub fn as_ref() -> &'static WorldBarrierData {
    BARRIERS_WORLD.as_ref()
}

// Internal
pub struct WorldBarrierDataInternal {
    // Frame syncing
    pub frame_sync_barrier: Barrier,
    pub special_frame_sync_barriers: HashMap<ThreadId, Barrier>,
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
    pub fn new_update(&self, thread_ids: Vec<ThreadId>) {
        let mut writer_ = self.internal.write().unwrap();
        let writer = &mut *writer_;
        let len = thread_ids.len();
        *writer = Some(WorldBarrierDataInternal {
            frame_sync_barrier: Barrier::new(len + 1),
            special_frame_sync_barriers: thread_ids.into_iter().map(|x| (x, Barrier::new(2))).collect::<HashMap<ThreadId, Barrier>>(),
            quit_loop_sync_barrier: Barrier::new(len + 2),
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
    // This is called at the start of the main thread frame, we wait until all the systems have synched up, so we can run all of them in parallel
    pub fn thread_sync(&self) {
        let r = &self.internal.read().unwrap();
        let frame_sync_barrier = &r.as_ref().unwrap().frame_sync_barrier;
        (frame_sync_barrier).wait();
    }
    // Sync up the main thread with this thread, at the end of the frame, so that system can access world_mut
    pub fn thread_sync_local_callbacks(&self, thread_id: &ThreadId) {
        let r = &self.internal.read().unwrap();
        let special_frame_sync_barriers = r.as_ref().unwrap().special_frame_sync_barriers.get(thread_id).unwrap();
        (special_frame_sync_barriers).wait();
    }
    // Sync up the quit barrier
    pub fn thread_sync_quit(&self) {
        let r = &self.internal.read().unwrap();
        let quit_loop_sync_barrier = &r.as_ref().unwrap().quit_loop_sync_barrier;
        (quit_loop_sync_barrier).wait();
    }
}
