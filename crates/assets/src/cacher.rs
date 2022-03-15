use crate::metadata::AssetMetadata;
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

// Half-assed multithreaded asset loading lol
lazy_static! {
    static ref CACHER: Mutex<AssetCacher> = Mutex::new(AssetCacher::default());
}
// Get the asset cacher
pub fn cacher() -> MutexGuard<'static, AssetCacher> {
    CACHER.lock().unwrap()
}

// Cacher that keeps assets loaded in, so it's cheaper to load them later
#[derive(Default)]
pub struct AssetCacher {
    cached: AHashMap<AssetMetadata, Vec<u8>>,
    asset_dir: Option<PathBuf>,
}

impl AssetCacher {
    // Cache an asset for later
    pub(crate) fn cache(&mut self, meta: AssetMetadata, bytes: Vec<u8>) {
        self.cached.insert(meta, bytes);
    }
    // Try to load a cached asset
    pub(crate) fn try_load(&self, meta: &AssetMetadata) -> Option<&[u8]> {
        self.cached.get(meta).map(|x| x.as_slice())
    }
    // Init the manager using a specific asset dir path
    pub fn init(&mut self, path: &str) {
        // Keep track of the directory containing the assets
        self.asset_dir.get_or_insert_with(|| Path::new(path).to_path_buf());
    }
    // Get the path that contains all the user assets
    pub fn get_user_assets_path(&self) -> Option<&PathBuf> {
        self.asset_dir.as_ref()
    }
    // Cache a persistent asset
    pub fn cache_persistent(&mut self, path: &str, bytes: Vec<u8>) -> &[u8] {
        println!("{:?}", path);
        let path = path.split("assets/").last().unwrap();
        let meta = AssetMetadata::new(path).unwrap();
        self.cached.entry(meta).or_insert(bytes)
    }
}
