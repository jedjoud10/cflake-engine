use std::collections::HashMap;

use crate::AssetMetadataLoadError;

// Caches the embeded bytes into an array basically
pub struct AssetCacher {
    // The cached metadata
    pub cached_metadata: HashMap<String, AssetMetadata>
}

impl AssetCacher {
    // Pre-load some asset metadata
    pub fn pre_load(&mut self, name: &str, bytes: &[u8], load_type: AssetLoadType) -> Result<(), AssetMetadataLoadError> {
        let data = AssetMetadata {
            bytes: bytes.to_vec(),
            load_type,
        };
        self.cached_metadata.insert(name.to_string(), data);
        Ok(())
    }
    // Load asset metadata
    pub fn load_md(&self, name: &str) -> Result<&AssetMetadata, AssetMetadataLoadError> {
        // Load
        let data = self.cached_metadata.get(name).ok_or(AssetMetadataLoadError::new_str("Asset was not pre-loaded!"))?;
        return Ok(data);
    }
    // Unload asset metadata (if possible)
    pub fn unload(&mut self, name: &str) -> Result<AssetMetadata, AssetMetadataLoadError> {
        // Check the load type
        let _type = self.cached_metadata.get(name).ok_or(AssetMetadataLoadError::new_str("Asset is not loaded in the first place!"))?;
        match &_type.load_type {
            AssetLoadType::Dynamic => {
                // Unload
                return Ok(self.cached_metadata.remove(name).unwrap());
            },
            _ => { /* Nothing */}
        }
        Err(AssetMetadataLoadError::new_str("No"))
    }
}

// For how long will this asset be alive?
pub enum AssetLoadType {
    Static, // You can only load it, you can't unload it
    Dynamic, // You can load it, and you can also unload it
    CustomCached, // Dispose of the bytes data, since the asset is customly cached
}
// Some data
pub struct AssetMetadata {
    // Bytes
    pub bytes: Vec<u8>,
    // Doodoo water
    pub load_type: AssetLoadType
}
impl AssetMetadata {
    // Turn the bytes into a UTF8 string
    pub fn read_string(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }
}
// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset from metadata
    fn load(data: &AssetMetadata) -> Self where Self: Sized; 
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_t(self, data: &AssetMetadata) -> Self where Self: Sized {
        panic!()
    }       
}