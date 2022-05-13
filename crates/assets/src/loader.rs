use crate::{Asset, CompoundAsset};
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
    instant_bytes_cached: Instant,

    // The total number of times this asset's bytes' been access to load it or to construct a compound asset
    num_times_accessed: u64,
}

impl Meta {
    // Get the path for the meta
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Get the moment we cached the bytes
    pub fn first_cache_instant(&self) -> &Instant {
        &self.instant_bytes_cached
    }

    // Get the number of times this asset's been loaded
    pub fn accessed(&self) -> u64 {
        self.num_times_accessed
    }
}


// Asset metadata for compound assets
pub struct CompoundMeta {
    // The paths used by each asset
    paths: Vec<PathBuf>,
}

impl CompoundMeta {
    // The paths used to load each asset
    pub fn paths(&self) -> &[PathBuf] {
        self.paths.as_slice()
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
        // Check if the extension is valid
        let path = PathBuf::from_str(path).unwrap();
        let extension = path.extension().and_then(OsStr::to_str)?;
        (!A::extensions().contains(&extension)).then(|| ())?;

        // If we have no bytes currently cached, try to load and cache them
        if self.cached.get(&path).is_none() {
            let bytes = super::raw::read(&path, &self.global)?;
            self.cached.insert(path.clone(), (bytes, Instant::now(), 0));
        };

        // Load the cached bytes and increment the accessed counter
        let (vec, instant, count) = self.cached.get_mut(&path)?;
        *count += 1;

        // Construct meta
        let meta = Meta {
            path,
            instant_bytes_cached: *instant,
            num_times_accessed: *count,
        };
        
        // Deserialize the asset
        Some(A::deserialize(vec.as_slice(), args, meta))
    }

    // Load an asset using some default loading arguments
    pub fn load<'loader, 'args, A: Asset<'args>>(&'loader mut self, path: &str) -> Option<A> where A::Args: Default {
        self.load_with(path, Default::default())
    }

    // Load a compound asset using some explicit loading arguments
    pub fn load_compound_with<'loader, 'args, A: CompoundAsset<'args>>(&'loader mut self, paths: &[&str], args: A::Args) -> Option<A> {
        // Convert the strings into pathbufs
        let paths = paths.iter().map(|path| PathBuf::from_str(path).ok()).collect::<Option<Vec<_>>>()?;
        
        // Convert all the extensions now
        let extensions = paths.iter().map(|a| a.extension().and_then(OsStr::to_str)).collect::<Option<Vec<_>>>()?;

        // Check if all the extensions are valid
        let all_path_extensions_valid = extensions.iter().all(|ext| A::extensions().contains(ext));
        
        // Convert the given paths into the corresponding bytes
        let tuples = paths.iter().map(|path| {
            // Cache the bytes if they are invalid
            let tuple = if self.cached.get(path).is_none() {                
                // Load & Cache
                let bytes = super::raw::read(&path, &self.global)?;
                self.cached.insert(path.clone(), (bytes, Instant::now(), 0));
                self.cached.get_mut(path)
            } else {
                // Load directly
                self.cached.get_mut(path)
            };

            // Increment the accessed counter while mapping
            tuple.map(|(vec, instant, count)| {
                *count += 1;
                vec.as_slice()
            })
        }).collect::<Option<Vec<_>>>()?;

        // Create the compound metadata now, since we have all the bytes and paths

        // Deserialize the asset
        Some(A::deserialize(slice, args, meta))
    }

    // Import a persistant asset using it's global asset path and it's raw bytes
    pub fn import(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path  = path.as_ref().strip_prefix("assets/").unwrap().to_path_buf();
        self.cached.entry(path).or_insert((bytes, Instant::now(), 0));
    }
}
