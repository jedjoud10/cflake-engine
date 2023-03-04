use crate::AssetLoadError;
use std::path::Path;

// If we are in Debug, we read the bytes directly from the file system
// Path is a global system wide path
pub fn read(
    path: &Path,
) -> Result<Vec<u8>, AssetLoadError> {
    use std::{fs::File, io::Read};

    // We do a bit of reading
    let mut file = File::open(path).ok().ok_or_else(|| {
        let path = path.as_os_str().to_str().unwrap().to_owned();
        AssetLoadError::DynamicNotFound(path)
    })?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok().unwrap();
    Ok(bytes)
}