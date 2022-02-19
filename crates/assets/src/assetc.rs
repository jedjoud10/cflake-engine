use crate::{asset::Asset, error::AssetLoadError, global::cacher, metadata::AssetMetadata};
use std::{fs::File, io::BufReader, path::PathBuf};
// If we are in Debug, we read the bytes directly from the source
#[cfg(debug_assertions)]
fn read_bytes(path: &str) -> Result<Vec<u8>, AssetLoadError> {
    // Open the source file direcetly and read
    use std::{env, io::BufRead, path::Path};
    // Get the path
    let file_path = {
        let mut file_path = env::current_dir().unwrap();
        file_path.push(Path::new("assets"));
        file_path.push(Path::new(path));
        file_path
    };
    dbg!(&file_path);
    let file = File::open(file_path).map_err(|_| AssetLoadError::new(path))?;
    // Read bytes
    let mut reader = BufReader::new(file);
    let bytes = reader.fill_buf().unwrap();
    Ok(bytes.to_vec())
}
// If we are in Release, we read the bytes from the "packed_assets" directory
#[cfg(not(debug_assertions))]
fn read_bytes(path: &str) -> Result<Vec<u8>, AssetLoadError> {
    /*
    // Open the file and read
    let file = File::open(file_path).map_err(|_| AssetLoadError::new(path))?;
    let reader = BufReader::new(file);
    let bytes = reader.buffer();
    */
    Ok(Vec::new())
}

// Read the bytes from an asset file and cache them if needed
// Path is the asset path relative to the "assets" directory
// Ex: "assets/user/trainingdata/test01.txt"
// Would be: path = "user/trainingdata/test01.txt"
// Load an asset
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
    obj.deserialize(&meta, bytes).ok_or_else(|| AssetLoadError::new(path))
}
// Load an asset (By creating a default version of it)
pub fn load<T: Asset + Default>(path: &str) -> Result<T, AssetLoadError> {
    load_with(path, T::default())
}
