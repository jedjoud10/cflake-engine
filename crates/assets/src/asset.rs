use crate::{metadata::AssetMetadata, error::AssetLoadError, cacher::cacher};
use std::path::PathBuf;
use std::fs::File;

// If we are in Debug, we read the bytes directly from the file system
#[cfg(debug_assertions)]
fn read_bytes(path: &str, asset_dir_path: PathBuf) -> Result<Vec<u8>, AssetLoadError> {
    // Open the source file directly and read
    use std::{io::Read, path::Path};
    // Get the path
    let file_path = {
        let mut file_path = asset_dir_path;
        file_path.push(Path::new(path));
        dbg!(&file_path);
        file_path
    };
    let mut file = File::open(file_path).map_err(|_| AssetLoadError::new(path))?;
    // Read bytes
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    Ok(bytes)
}
// If we are in Release, we read the bytes from the cacher directly since they are embedded into the binary
#[cfg(not(debug_assertions))]
fn read_bytes(path: &str, _asset_dir_path: PathBuf) -> Result<Vec<u8>, AssetLoadError> {
    Err(AssetLoadError::new(&format!("The asset '{}' is not cached!", path)))
}

// An asset loader
pub trait Asset: where Self: Default + Sized {
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_raw(meta: &AssetMetadata, bytes: &[u8]) -> Option<Self>;
    // Load an asset
    fn load(path: &str) -> Result<Self, AssetLoadError> {
        // Create metadata
        let meta = AssetMetadata::new(path).unwrap();
        // Load bytes
        let mut cacher = cacher();
        // Try to load the cached object bytes
        let bytes = if let Some(cached) = cacher.try_load(&meta) {
            cached
        } else {
            // Cache the bytes
            let asset_dir_path = cacher.get_user_assets_path().to_path_buf();
            cacher.cache(meta.clone(), read_bytes(path, asset_dir_path)?);
            cacher.try_load(&meta).unwrap()
        };
        // Deserialize
        Self::load_raw(&meta, bytes).ok_or_else(|| AssetLoadError::new(path))
    }
}

// Specific cache commands
pub mod caching {
    use crate::{cacher::cacher, metadata::AssetMetadata};

    // Un-cache an asset
    pub fn uncache(path: &str) {
        let mut cacher = cacher();
        cacher.uncache(AssetMetadata::new(path).unwrap());
    }
}
