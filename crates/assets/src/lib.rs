// Export
mod asset_cacher;
mod asset_object;
mod commands;
mod error;
mod macros;
mod tests;
use std::sync::Mutex;

pub use asset_cacher::*;
pub use asset_object::*;
pub use commands::*;
pub use error::*;
pub use macros::*;

// Asset Loading and Object Caching Commands
pub mod globals {
    use std::sync::MutexGuard;

    use crate::AssetCacher;
    use crate::*;
    use lazy_static::lazy_static;
    // Half-assed multithreaded rendering lol
    lazy_static! {
        static ref ASSETM: Mutex<AssetCacher> = Mutex::new(AssetCacher::default());
    }
    // Get the asset cacher
    pub fn asset_cacher() -> MutexGuard<'static, AssetCacher> {
        ASSETM.lock().unwrap()
    }
}
