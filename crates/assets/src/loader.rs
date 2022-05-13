use crate::{Asset};
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, MutexGuard}, time::Instant,
};

// Asset metadata that contains some data about how and when we loaded the asset
pub struct Meta {
    // The path of the loaded asset
    path: PathBuf,

    // The exact moment we cached the bytees
    cached: Instant,

    // The total number of times this asset's been loaded
    count: u64,
}

impl Meta {
    // Get the path for the meta
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Get the moment we cached the bytes
    pub fn first_cache_instant(&self) -> &Instant {
        &self.cached
    }

    // Get the number of times this asset's been loaded
    pub fn count(&self) -> u64 {
        self.count
    }
}


// Asset manager that will cache all the assets and help us load them in
pub struct AssetLoader {
    // Byte caching (the key is the relative path of the asset)
    cached: AHashMap<PathBuf, (Vec<u8>, Instant, u64)>,

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

    // Load an asset using some explicit loading arguments
    pub fn load_with<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str, args: A::Args) -> Option<A> {
        // Check if the extension is valid, and return None if it doesn't validate any of the extensions for the asset
        let path = PathBuf::from_str(path).unwrap();
        let extension = path.extension().and_then(OsStr::to_str)?;
        (!A::extensions().contains(&extension)).then(|| ())?;

        // Load the bytes from the file if they don't exist
        if self.cached.get(&path).is_none() {
            // Cache the bytes if needed (but split the path)
            let bytes = super::raw::read(&path, &self.global)?;
            self.cached.insert(path.clone(), (bytes, Instant::now(), 0));
        }

        // Create some asset metadata (and a valid byte slice) using the asset path
        let (slice, meta) = self.cached.get_mut(&path).map(|(vec, instant, count)| {
            let slice = vec.as_slice();
            *count += 1;
            
            (slice, Meta {
                path,
                cached: *instant,
                count: *count,
            })
        })?;

        // Deserialize the asset
        Some(A::deserialize(slice, args, meta))
    }

    // Load an asset using some default loading arguments
    pub fn load<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Option<A> where A::Args: Default {
        self.load_with(path, Default::default())
    }

    // Cache an asset manually, given it's path and it's bytes
    pub fn import(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path  = path.as_ref().strip_prefix("assets/").unwrap().to_path_buf();
        self.cached.entry(path).or_insert((bytes, Instant::now(), 0));
    }
}
