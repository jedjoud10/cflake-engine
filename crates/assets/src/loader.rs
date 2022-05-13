use crate::{Asset};
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, MutexGuard},
};

// Dis dumb but it works
// TODO: Rename
pub type LoadedData<'l, 'args, A: Asset<'args>> = (&'l [u8], A::Args, PathBuf);


// Asset manager that will cache all the assets and help us load them in
pub struct AssetLoader {
    // Byte caching (the key is the relative path of the asset)
    cached: AHashMap<PathBuf, Vec<u8>>,

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
            self.cached.insert(path.clone(), bytes);
        }

        // Make sure to only get a slice of the bytes, and not the whole vec
        let slice = self.cached.get(&path).map(Vec::as_slice)?;

        // Deserialize the asset
        Some(A::deserialize((
            slice,
            args,
            path,
        )))
    }

    // Load an asset using some default loading arguments
    pub fn load<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Option<A> where A::Args: Default {
        self.load_with(path, Default::default())
    }

    // Cache an asset manually, given it's path and it's bytes
    pub fn import(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path  = path.as_ref().strip_prefix("assets/").unwrap().to_path_buf();
        self.cached.entry(path).or_insert(bytes);
    }
}
