use crate::AssetLoadError;
use std::collections::HashMap;

// Caches the embeded bytes into an array basically
#[derive(Default)]
pub struct AssetCacher {
    // The cached metadata
    pub cached_metadata: HashMap<String, AssetMetadata>,
}

impl AssetCacher {
    // Pre-load some asset metadata
    pub fn pre_load(&mut self, name: &str, bytes: &[u8]) -> Result<(), AssetLoadError> {
        let name = name.split("resources\\").last().unwrap();
        // Get the extension
        let first_dot_index = name.split("").position(|c| c == ".").unwrap();
        let extension = name.split_at(first_dot_index).1.to_string();

        let data = AssetMetadata {
            bytes: bytes.to_vec(),
            load_type: AssetLoadType::Dynamic,
            extension,
            name: name.to_string(),
        };
        self.cached_metadata.insert(name.to_string(), data);
        Ok(())
    }
}

// For how long will this asset be loaded?
#[derive(Debug, Clone, Copy)]
pub enum AssetLoadType {
    Static,  // You can only load it, you can't unload it
    Dynamic, // You can load it, and you can also unload it
    Manual,  // Dispose of the bytes data, since the asset is manually cached
}
// Some data
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    // The bytes that were loaded in whilst loading the asset metadata
    pub bytes: Vec<u8>,
    // How should we manage the disposing of this asset?
    pub load_type: AssetLoadType,
    // The extension of the path of the asset
    pub extension: String,
    // The name of the asset. PS: This also contains the extension
    pub name: String,
}
impl AssetMetadata {
    // Turn the bytes into a UTF8 string
    pub fn read_string(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }
}
// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_medadata(self, data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized;
}
