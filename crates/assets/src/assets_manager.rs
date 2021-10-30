use std::{collections::HashMap, fs::Metadata};

use crate::{AssetLoadError, AssetLoadType, AssetMetadata};

// Asset manager
pub struct AssetManager {
    // Cached bytes
    pub cached_metadata: HashMap<String, AssetMetadata>,
}

impl AssetManager {
    // Pre-load some asset bytes
    pub fn pre_load(&mut self, name: &str, bytes: &[u8], load_type: AssetLoadType) -> Result<(), AssetLoadError> {
        let data = AssetMetadata {
            bytes: bytes.to_vec(),
            load_type,
        };
        self.cached_metadata.insert(name.to_string(), data);
        Ok(())
    }
    // Load an asset
    pub fn load(&self, name: &str) -> Result<&AssetMetadata, AssetLoadError> {
        // Load
        let data = self.cached_metadata.get(name).ok_or(AssetLoadError::new_str("Asset was not pre-loaded!"))?;
        return Ok(data);
    }
    // Unload an asset (if possible)
    pub fn unload(&mut self, name: &str) -> Result<AssetMetadata, AssetLoadError> {
        // Check the load type
        let _type = self.cached_metadata.get(name).ok_or(AssetLoadError::new_str("Asset is not loaded in the first place!"))?;
        match &_type.load_type {
            AssetLoadType::Dynamic => {
                // Unload
                return Ok(self.cached_metadata.remove(name).unwrap());
            },
            _ => { /* Nothing */}
        }
        Err(AssetLoadError::new_str("No"))
    }
}