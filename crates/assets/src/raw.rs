use std::path::{Path};

// If we are in Debug, we read the bytes directly from the file system
#[cfg(debug_assertions)]
pub fn read(path: &Path, asset_dir_path: &Path) -> Option<Vec<u8>> {
    use std::{fs::File, io::Read};

    // Get the path of the file (global)
    let file_path = {
        let mut file_path = asset_dir_path.to_path_buf();
        file_path.push(path);
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
pub fn read(path: &Path, _asset_dir_path: &Path) -> Option<Vec<u8>> {
    None
}
