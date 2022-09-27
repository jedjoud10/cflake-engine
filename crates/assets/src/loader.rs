use crate::Asset;
use ahash::AHashMap;
use parking_lot::RwLock;

use std::{
    cell::RefCell,
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

// This is the main asset manager resource that will load & cache newly loaded assets
// This asset manager will also contain the persistent assets that are included by default into the engine executable
// TODO: Test multithreadeding
pub struct Assets {
    cached: RwLock<AHashMap<PathBuf, Arc<[u8]>>>,

    // Global path to the user defined assets folder
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

    // Load an asset using some explicit loading arguments without checking it's extensions
    pub unsafe fn load_with_unchecked<'args, A: Asset<'args>>(
        &self,
        path: &str,
        args: A::Args,
    ) -> Option<A> {
        // Check if the extension is valid
        let path = PathBuf::from_str(path).unwrap();
        let (name, extension) = path.file_name().and_then(OsStr::to_str)?.split_once('.')?;

        // If we have no bytes currently cached, try to load and cache them
        if !self.cached.read().contains_key(&path) {
            let bytes = super::raw::read(path.as_path(), self.user.as_ref()?)?;
            let index = self.cached.read().len();
            self.cached.write().insert(path.clone(), Arc::from(bytes));
        };

        // Load the cached bytes and increment the accessed counter
        let slice = self.cached.read().get(&path).map(Arc::clone)?;

        // Deserialize the asset file
        Some(A::deserialize(
            crate::Data {
                name,
                extension,
                bytes: slice,
                path: &path,
                loader: self,
            },
            args,
        ))
    }

    // Load an asset using some explicit loading arguments
    pub fn load_with<'args, A: Asset<'args>>(&self, path: &str, args: A::Args) -> Option<A> {
        // Check if the extension is valid
        let _path = PathBuf::from_str(path).unwrap();
        let (_, extension) = _path.file_name().and_then(OsStr::to_str)?.split_once('.')?;

        // If the asset has no extensions, we shall not check
        ((A::extensions().contains(&extension)) || A::extensions().is_empty()).then_some(())?;
        unsafe { self.load_with_unchecked(path, args) }
    }

    // Load an asset using some default loading arguments
    pub fn load<'args, A: Asset<'args>>(&self, path: &str) -> Option<A>
    where
        A::Args: Default,
    {
        self.load_with(path, Default::default())
    }

    // Import a persistent asset using it's global asset path and it's raw bytes
    pub fn import(&self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path = path
            .as_ref()
            .strip_prefix("./assets/")
            .unwrap()
            .to_path_buf();
        dbg!(&path);
        self.cached.write().entry(path).or_insert(Arc::from(bytes));
    }
}
