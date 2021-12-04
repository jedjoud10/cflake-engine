use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Global main for purely just low level task management
use lazy_static::lazy_static;

use crate::World;
lazy_static! {
    static ref WORLD: Arc<RwLock<World>> = Arc::new(RwLock::new(crate::new("NullDev", "NullGame")));
}

pub fn world() -> RwLockReadGuard<'static, World> {
    let x = WORLD.as_ref().read().unwrap();
    x
}

pub fn world_mut() -> RwLockWriteGuard<'static, World> {
    let x = WORLD.as_ref().write().unwrap();
    x
}