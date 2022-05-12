use crate::{Asset, LoadError};
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, MutexGuard},
};

// If we are in Debug, we read the bytes directly from the file system
#[cfg(debug_assertions)]
fn read(path: &str, asset_dir_path: &PathBuf) -> Option<Vec<u8>> {
    use std::{fs::File, io::Read, path::Path};

    // Get the path of the file (global)
    let file_path = {
        let mut file_path = asset_dir_path.clone();
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
fn read(path: &str, _asset_dir_path: &PathBuf) -> Option<Vec<u8>> {
    None
}

// Simple loading context to avoid loading assets unsafely
pub struct LoadingContext(());

// Asset manager that will cache all the assets and help us load them in
pub struct AssetLoader {
    // Byte caching (the key is the relative path of the asset)
    cached: AHashMap<String, Vec<u8>>,

    // Global assets path
    global: PathBuf,
}

impl AssetLoader {
    // Create a new asset loader using a path to an asset folder
    pub fn new(path: &str) -> Self {
        Self {
            cached: Default::default(),
            global: PathBuf::from_str(path).unwrap(),
        }
    }

    // Load bytes from a path, and make sure we have the specific file loaded in
    // Ex: "defaults/meshes/cube.obj"
    pub fn load_bytes<'loader, 'path>(&'loader mut self, path: &'path str) -> Option<&'loader [u8]> {
        if self.cached.get(path).is_none() {
            // Cache the bytes if needed (but split the path)        
            let bytes = read(path, &self.global)?;
            self.cached.insert(path.to_string(), bytes);
        }

        // Make sure to only get a slice of the bytes, and not the whole vec
        self.cached.get(path).map(|vec| vec.as_ref())
    }

    // Import an asset during compile time
    pub fn import(&mut self, path: &str, bytes: Vec<u8>) {
        let path = path.split("assets/").last().unwrap();
        self.cached.entry(path.to_string()).or_insert(bytes);
    }

    // Load 
}