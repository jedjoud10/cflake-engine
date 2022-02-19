use std::{ffi::OsString, path::Path};

// Asset metadata that contains the name and extension of an asset
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct AssetMetadata {
    // Paths
    pub extension: OsString,
    // PS: This also contains the extension
    pub name: OsString,
    pub relative_path: OsString,
}

impl AssetMetadata {
    // Create some new asset metadata using a path
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        // Get the name of the asset
        let path = path.as_ref();
        let extension = path.extension()?.to_os_string();
        let name = path.file_name()?.to_os_string();
        Some(Self {
            extension,
            name,
            relative_path: path.as_os_str().to_os_string(),
        })
    }
}
