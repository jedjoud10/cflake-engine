use crate::Asset;
use ahash::AHashMap;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};
use world::resources::Resource;

// Idk how many times I've rewritten this. Help my sanity. 12:16am on the 14th may 2022
pub struct CachedSlice<'loader>(&'loader [u8]);

impl<'loader> AsRef<[u8]> for CachedSlice<'loader> {
    fn as_ref(&self) -> &'loader [u8] {
        self.0
    }
}

// This is the main asset manager resource that will load & cache newly loaded assets
// This asset manager will also contain the persistent assets that are included by default into the engine executable
#[derive(Resource)]
#[Locked]
pub struct Assets {
    // Byte caching (the key is the relative path of the asset)
    cached: AHashMap<PathBuf, Vec<u8>>,

    // GLobal path to the user defined assets folder
    user: Option<PathBuf>,
}

impl Assets {
    // Create a new asset loader using a path to the user defined asset folder (if there is one)
    pub fn new(user: Option<PathBuf>) -> Self {
        Self {
            cached: Default::default(),
            user,
        }
    }

    // Load an asset using some explicit loading arguments
    pub fn load_with<'loader, 'args, A: Asset<'args>>(
        &'loader mut self,
        path: &str,
        args: A::Args,
    ) -> Option<A> {
        // Check if the extension is valid
        let path = PathBuf::from_str(path).unwrap();
        let (_name, extension) = path.file_name().and_then(OsStr::to_str)?.split_once('.')?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension)) || A::extensions().is_empty()).then(|| ())?;

        // If we have no bytes currently cached, try to load and cache them
        if self.cached.get(&path).is_none() {
            let bytes = super::raw::read(&path, self.user.as_ref()?)?;
            self.cached.insert(path.clone(), bytes);
        };

        // Load the cached bytes and increment the accessed counter
        let slice = self.cached.get(&path).map(Vec::as_slice)?;

        // Deserialize the asset
        Some(A::deserialize(CachedSlice(slice), args))
    }

    // Load an asset using some default loading arguments
    pub fn load<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Option<A>
    where
        A::Args: Default,
    {
        self.load_with(path, Default::default())
    }

    // Import a persistant asset using it's global asset path and it's raw bytes
    pub fn import(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path = path.as_ref().strip_prefix("./assets/").unwrap().to_path_buf();
        self.cached.entry(path).or_insert(bytes);
    }
}
