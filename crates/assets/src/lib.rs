// Export
mod asset_cacher;
mod asset_object;
mod commands;
mod error;
mod macros;
mod object_cacher;
mod tests;
use std::sync::Mutex;

pub use asset_cacher::*;
pub use asset_object::*;
pub use commands::*;
pub use error::*;
pub use macros::*;
pub use object_cacher::*;

// Asset Loading and Object Caching Commands
pub mod globals {
    use std::sync::MutexGuard;

    use crate::AssetCacher;
    use crate::*;
    use lazy_static::lazy_static;
    // Half-assed multithreaded rendering lol
    lazy_static! {
        static ref ASSETM: no_deadlocks::Mutex<AssetCacher> = no_deadlocks::Mutex::new(AssetCacher::default());
        static ref OBJECTM: no_deadlocks::Mutex<ObjectCacher> = no_deadlocks::Mutex::new(ObjectCacher::default());
    }
    // Get the asset cacher
    pub fn asset_cacher() -> no_deadlocks::MutexGuard<'static, AssetCacher> {
        ASSETM.lock().unwrap()
    }
    // Get the object cacher
    pub fn object_cacher() -> no_deadlocks::MutexGuard<'static, ObjectCacher> {
        OBJECTM.lock().unwrap()
    }
}
