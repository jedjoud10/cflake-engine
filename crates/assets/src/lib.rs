// Export
mod asset_cacher;
mod asset_object;
mod assets_manager;
mod error;
mod macros;
mod object_cacher;
mod tests;
mod commands;
use std::sync::Mutex;

pub use asset_cacher::*;
pub use asset_object::*;
pub use assets_manager::*;
pub use error::*;
pub use macros::*;
pub use object_cacher::*;
pub use commands::*;

// Asset Loading and Object Caching Commands
mod main {
    use std::sync::MutexGuard;

    use crate::AssetCacher;
    use crate::*;
    use lazy_static::lazy_static;
    // Half-assed multithreaded rendering lol
    lazy_static! {
        static ref ASSETM: Mutex<AssetCacher> = Mutex::new(AssetCacher::default());
        static ref OBJECTM: Mutex<ObjectCacher> = Mutex::new(ObjectCacher::default());
    }
    // Get the asset cacher
    #[inline]
    pub fn asset_cacher() -> MutexGuard<'static, AssetCacher> {
        println!("AA");
        ASSETM.lock().unwrap()
    }
    // Get the object cacher
    #[inline]
    pub fn object_cacher() -> MutexGuard<'static, ObjectCacher> {
        OBJECTM.lock().unwrap()
    }
}
