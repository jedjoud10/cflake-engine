use crate::{asset::Asset, cacher::*, error::AssetLoadError, metadata::AssetMetadata};
use std::fs::File;
// If we are in Debug, we read the bytes directly from the source
#[cfg(debug_assertions)]
fn read_bytes(path: &str) -> Result<Vec<u8>, AssetLoadError> {
    // Open the source file directly and read
    use std::{env, io::Read, path::Path};
    // Get the path
    let file_path = {
        // I hate living
        let manifest_path = env!("CARGO_MANIFEST_DIR");
        let mut file_path = Path::new(manifest_path).to_path_buf();
        file_path.pop();
        file_path.pop();
        file_path.push(Path::new("assets"));
        file_path.push(Path::new(path));
        file_path
    };
    let mut file = File::open(file_path).map_err(|_| AssetLoadError::new(path))?;
    // Read bytes
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    Ok(bytes)
}
// If we are in Release, we read the bytes from the cacher directly since they are embedded into the engine
#[cfg(not(debug_assertions))]
fn read_bytes(path: &str) -> Result<Vec<u8>, AssetLoadError> {
    Err(AssetLoadError::new("The asset should already be cached!"))
}

// Read the bytes from an asset file and cache them if needed
pub fn load_with<T: Asset>(path: &str, obj: T) -> Result<T, AssetLoadError> {
    // Create metadata
    let meta = AssetMetadata::new(path).unwrap();
    // Load bytes
    let mut cacher = cacher();
    // Try to load the cached object bytes
    let bytes = if let Some(cached) = cacher.try_load(&meta) {
        cached
    } else {
        // Cache the bytes
        cacher.cache(meta.clone(), read_bytes(path)?);
        cacher.try_load(&meta).unwrap()
    };
    // Deserialize
    obj.deserialize(&meta, bytes)
        .ok_or_else(|| AssetLoadError::new(path))
}
// Load an asset (By creating a default version of it)
pub fn load<T: Asset + Default>(path: &str) -> Result<T, AssetLoadError> {
    load_with(path, T::default())
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
