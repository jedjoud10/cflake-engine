use crate::{asset::Asset, cacher::*, error::AssetLoadError, metadata::AssetMetadata};
use std::{fs::File, path::PathBuf};

// TODO: Rewrite this and add async asset loader

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

// Read the bytes from an assert file and cache them if needed
fn read(path: &str) -> Result<(&'static [u8], AssetMetadata), AssetLoadError> {
    // Create metadata
    let meta = AssetMetadata::new(path).unwrap();
    // Load bytes
    let mut cacher = cacher();
    // Try to load the cached object bytes
    eprintln!("Loading '{}'...", path);
    let bytes = if let Some(cached) = cacher.try_load(&meta) {
        cached
    } else {
        // Cache the bytes
        let asset_dir_path = cacher
            .get_user_assets_path()
            .ok_or(AssetLoadError::new("The asset cacher was not initialized!"))?
            .to_path_buf();
        cacher.cache(meta.clone(), read_bytes(path, asset_dir_path)?);
        cacher.try_load(&meta).unwrap()
    };

    // Fuck this shit, basically
    let bytes = unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) };
    Ok((bytes, meta))
}

// Load an asset by creating it's input from default
pub fn load<T: Asset>(path: &str) -> Result<T, AssetLoadError>
where
    T::Input: Default,
{
    load_with(path, T::Input::default())
}

// Load an asset with an explicity load input
pub fn load_with<T: Asset>(path: &str, i: T::Input) -> Result<T, AssetLoadError> {
    let (bytes, meta) = read(path)?;
    T::deserialize(&meta, bytes, i).ok_or_else(|| AssetLoadError::new(path))
}
