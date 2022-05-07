use crate::{metadata::AssetMetadata, Asset};
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

// If we are in Debug, we read the bytes directly from the file system
#[cfg(debug_assertions)]
fn read(path: &str, asset_dir_path: PathBuf) -> Option<Vec<u8>> {
    use std::{io::Read, path::Path, fs::File};
    
    // Get the path of the file (global)
    let file_path = {
        let mut file_path = asset_dir_path;
        file_path.push(Path::new(path));
        dbg!(&file_path);
        file_path
    };
    
    // We do a bit of reading
    let mut file = File::open(file_path).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

// If we are in Release, we read the bytes from the cacher directly since they are embedded into the binary
#[cfg(not(debug_assertions))]
fn read(path: &str, _asset_dir_path: PathBuf) -> Option<Vec<u8>> {
    None
}

// Asset manager that will cache all the assets and help us load them in
pub struct AssetLoader {
    // Byte caching
    cached: AHashMap<AssetMetadata, Vec<u8>>,

    // Global assets path
    global: PathBuf,
}

impl AssetLoader {
    // Try to load an asset with some explicit optional arguments
    pub fn load_with<A: Asset, P: AsRef<PathBuf>>(&self, path: P, args: A::OptArgs) -> T {
        // Try to load some cached bytes, if possible
        let meta = AssetMetadata::new(path).unwrap();
        let cached = self.cached.get(&meta);

        if let Some(cached) = cached {
            // Deserialize the asset using the cached bytes
            let asset = A::deserialize(&meta, &cached, args);
        } else {
            // Deserialize the asset using new bytes that we will load in
            let bytes = read(path.)
        }
    } 


    // Import an asset during compile time
    pub fn import(&mut self, path: &str, bytes: Vec<u8>) -> &[u8] {
        let path = path.split("assets/").last().unwrap();
        let meta = AssetMetadata::new(path).unwrap();
        self.cached.entry(meta).or_insert(bytes)
    }
}
