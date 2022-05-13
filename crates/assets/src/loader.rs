use crate::{Asset, CompoundAsset};
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, MutexGuard},
};

// Bytes that the asset loader will use to deserialize assets
pub struct AssetBytes<'loader>(pub(crate) &'loader [u8]);

// This forces us to pass through the Asset::validate_bytes
impl<'a> AsRef<[u8]> for AssetBytes<'a> {
    fn as_ref(&self) -> &'a [u8] {
        self.0
    }
}

// This is given to CompoundAssets to be able to deserialize them
pub struct CompoundLoadingContext<'loader>(&'loader mut AssetLoader);

impl<'loader> AsMut<AssetLoader> for CompoundLoadingContext<'loader> {
    fn as_mut(&mut self) -> &mut AssetLoader {
        self.0
    }
}

impl<'loader> AsRef<AssetLoader> for CompoundLoadingContext<'loader> {
    fn as_ref(&self) -> &AssetLoader {
        self.0
    }
}

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

    // Load an asset by deserializing it's bytes using some explicit loading arguments
    pub fn try_load_with<'loader, 'args, A: Asset<'args>>(&'loader mut self, args: A::Args, path: &str) -> Option<A> {
        // Check if the extension is valid, and return None if it doesn't validate any of the extensions for the asset
        let path = PathBuf::from_str(path).unwrap();
        let extension = path.extension().and_then(OsStr::to_str)?;
        if A::extensions().contains(&extension) {
            return None;
        }

        // Load the bytes from the file if they don't exist
        if self.cached.get(&path).is_none() {
            // Cache the bytes if needed (but split the path)
            let bytes = super::raw::read(&path, &self.global)?;
            self.cached.insert(path.clone(), bytes);
        }

        // Make sure to only get a slice of the bytes, and not the whole vec
        let slice = self.cached.get(&path).map(|vec| AssetBytes(vec.as_ref()))?;

        // Deserialize the asset
        Some(A::deserialize(slice, path, args))
    }

    // Load an asset by deserializing it's bytes and using some default loading arguments
    pub fn try_load<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Option<A>
    where
        A::Args: Default,
    {
        self.try_load_with(Default::default(), path)
    }

    // Load a compound asset given multiple paths and some explicit loading arguments
    pub fn try_load_compound_with<'loader, 'args, A: CompoundAsset<'args>>(&'loader mut self, args: A::Args, paths: &[&str]) -> Option<A> {
        // Check if all the extensions of each path are all valid
        let valid = paths
            .iter()
            .map(|path| Path::new(path).extension().and_then(OsStr::to_str))
            .filter_map(|a| a)
            .all(|extension| A::extensions().contains(&extension));

        // Sometimes, it can look beautiful
        valid.then(|| {
            // Just create the compound asset without loading anything automatically
            let context = CompoundLoadingContext(self);
            A::deserialize(context, args)
        })
    }

    // Cache an asset manually, given it's path and it's bytes
    pub fn import(&mut self, path: &str, bytes: Vec<u8>) {
        let path = PathBuf::from_str(path.split("assets/").last().unwrap()).unwrap();
        self.cached.entry(path).or_insert(bytes);
    }

    // Load a compound asset given multiple paths and some default loading arguments
    pub fn try_load_compound<'loader, 'args, A: CompoundAsset<'args>>(&'loader mut self, paths: &[&str]) -> Option<A>
    where
        A::Args: Default,
    {
        self.try_load_compound_with(Default::default(), paths)
    }
}
