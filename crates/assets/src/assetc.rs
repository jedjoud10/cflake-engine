use std::{fs::File, io::BufReader};

use crate::{asset::Asset, error::AssetLoadError, global::cacher, metadata::AssetMetadata};

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
    let bytes = if let Some(cached) = cacher.try_load(&meta) { cached }
    else {
        // We must load the asset bytes for the first time
        let mut file_path = cflake_engine_packer::get_assets_dir().map_err(|_| AssetLoadError::new(path))?;
        file_path.push(meta.relative_path.clone());
        // Open the file and read
        let file = File::open(file_path).map_err(|_| AssetLoadError::new(path))?;
        let reader = BufReader::new(file);
        let bytes = reader.buffer();
        // Cache the bytes
        cacher.cache(meta.clone(), bytes.to_vec());    
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