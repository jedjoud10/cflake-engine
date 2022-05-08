use crate::{metadata::AssetMetadata, Asset};
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

// Asset manager that will cache all the assets and help us load them in
pub struct AssetLoader {
    // Byte caching
    cached: RefCell<AHashMap<AssetMetadata, Vec<u8>>>,

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

    // Try to load an asset with some explicit optional arguments
    pub(crate) fn load_with<A: Asset>(&self, path: &str) -> Option<(AssetMetadata, Ref<[u8]>)> {
        // Try to load some cached bytes, if possible
        let path = PathBuf::from_str(path).unwrap();
        let meta = AssetMetadata::new(path.clone()).unwrap();

        // Cache the bytes if needed
        let mut borrowed = self.cached.borrow_mut();
        if borrowed.get(&meta).is_none() {
            // Cache the bytes
            let path = path.as_os_str().to_str()?;
            let bytes = read(path, &self.global)?;
            borrowed.insert(meta.clone(), bytes);
        }
        drop(borrowed);

        // And convert to the final tuple
        let mapped = Ref::map(self.cached.borrow(), |m| m.get(&meta).unwrap().as_slice());
        Some((meta.clone(), mapped))
    }

    // Import an asset during compile time
    pub fn import(&mut self, path: &str, bytes: Vec<u8>) {
        let path = path.split("assets/").last().unwrap();
        let meta = AssetMetadata::new(path).unwrap();
        self.cached.borrow_mut().entry(meta).or_insert(bytes);
    }
}

// Default asset implementations
impl Asset for String {
    type OptArgs = ();

    fn is_valid(meta: AssetMetadata) -> bool {
        true
    }

    unsafe fn deserialize(bytes: &[u8], args: &Self::OptArgs) -> Option<Self> {
        Self::from_utf8(bytes.to_vec()).ok()
    }
}

/*
impl crate::Asset for String {
    type OptArgs = ();
    const EXTENSION: &'static str = "";

    fn try_deserialize(meta: &crate::metadata::AssetMetadata, bytes: &[u8], args: &Self::OptArgs) -> Option<Self>
    where Self: Sized {
        Self::from_utf8(bytes.to_vec()).ok()
    }

    // We don't implement anything for desrialize, since the from_utf8 function is faillible, so it must be implemented for try_deserialize instead
    fn deserialize(bytes: &[u8], args: &Self::OptArgs) -> Self
    where
        Self: Sized {
        todo!()
    }
}
*/
