use std::sync::{Arc, RwLock};

// Global main for purely just low level task management
use lazy_static::lazy_static;

use crate::World;
lazy_static! {
    static ref WORLD: Arc<RwLock<World>> = Arc::new(RwLock::new(World::default()));
}