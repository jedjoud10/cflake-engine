use crate::{metadata::AssetMetadata, Asset, LoadError};
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
    cached: AHashMap<AssetMetadata, Vec<u8>>,

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
    pub(crate) fn load_with<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Result<(AssetMetadata, &'loader [u8]), LoadError> {
        // Try to load some cached bytes, if possible
        let pathbuf = PathBuf::from_str(path).unwrap();
        let path = pathbuf.as_os_str().to_str().unwrap();
        let meta = AssetMetadata::new(pathbuf).unwrap();

        // Cache vs fast load to fetch the bytes
        let bytes = if self.cached.get(&meta).is_none() {
            // Cache the bytes if needed            
            let bytes = read(path, &self.global).ok_or(LoadError::Invalid(path))?;
            self.cached.insert(meta.clone(), bytes);
            self.cached.get(&meta).unwrap().as_ref()
        } else {
            // Might fail, so we have to check the result
            self.cached.get(&meta).ok_or(LoadError::Invalid(path))?
        };
        
        // And convert to the final tuple
        Ok((meta.clone(), bytes.as_slice()))
    }

    // Import an asset during compile time
    pub fn import(&mut self, path: &str, bytes: Vec<u8>) {
        let path = path.split("assets/").last().unwrap();
        let meta = AssetMetadata::new(path).unwrap();
        self.cached.entry(meta).or_insert(bytes);
    }
}

// Default asset implementations
impl Asset<'static> for String {
    type OptArgs = ();

    fn is_valid(meta: AssetMetadata) -> bool {
        true
    }

    unsafe fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Option<Self> {
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
