use crate::cacher::AssetCacher;
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};
// Half-assed multithreaded asset loading lol
lazy_static! {
    static ref CACHER: Mutex<AssetCacher> = Mutex::new(AssetCacher::default());
}
// Get the asset cacher
pub fn cacher() -> MutexGuard<'static, AssetCacher> {
    CACHER.lock().unwrap()
}
