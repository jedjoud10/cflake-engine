use crate::{asset::Asset, cacher::*, error::AssetLoadError, metadata::AssetMetadata};
use std::{fs::File, path::PathBuf};
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

// Specific cache commands
pub mod caching {
    use crate::{cacher::cacher, metadata::AssetMetadata};

    // Un-cache an asset
    pub fn uncache(path: &str) {
        let mut cacher = cacher();
        cacher.uncache(AssetMetadata::new(path).unwrap());
    }
}
