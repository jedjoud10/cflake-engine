// Export
mod asset_cacher;
mod asset_object;
mod assets_manager;
mod error;
mod macros;
mod object_cacher;
use std::sync::Mutex;

pub use asset_cacher::*;
pub use asset_object::*;
pub use assets_manager::*;
pub use error::*;
pub use macros::*;
pub use object_cacher::*;

// Asset Loading and Object Caching Commands
pub mod alocc {
    use std::sync::MutexGuard;

    use crate::*; 
    use lazy_static::lazy_static;
    use crate::AssetCacher;
    // Half-assed multithreaded rendering lol
    lazy_static! {
        static ref ASSETM: Mutex<AssetCacher> = Mutex::new(AssetCacher::default());
        static ref OBJECTM: Mutex<ObjectCacher> = Mutex::new(ObjectCacher::default());
    }

    // Get the asset cacher
    pub fn asset_cacher() -> MutexGuard<'static, AssetCacher> {
        ASSETM.lock().unwrap()
    }
    // Get the object cacher
    pub fn object_cacher() -> MutexGuard<'static, ObjectCacher> {
        OBJECTM.lock().unwrap()
    }
    // Load a single asset
}