use std::{collections::HashMap, fs::Metadata, rc::Rc};
use crate::{Asset, AssetLoadError, AssetLoadType, AssetMetadata, CachedAsset};

// Asset manager
pub struct AssetManager {
    // The cached asset metadata
    pub cached_metadata: HashMap<String, AssetMetadata>,
    // The cached assets
    pub cached_assets: HashMap<String, CachedAsset>
}

impl AssetManager {
    // Pre-load some asset metadata
    pub fn pre_load(&mut self, name: &str, bytes: &[u8], load_type: AssetLoadType) -> Result<(), AssetLoadError> {
        let data = AssetMetadata {
            bytes: bytes.to_vec(),
            load_type,
        };
        self.cached_metadata.insert(name.to_string(), data);
        Ok(())
    }
    // Load asset metadata
    pub fn load_md(&self, name: &str) -> Result<&AssetMetadata, AssetLoadError> {
        // Load
        let data = self.cached_metadata.get(name).ok_or(AssetLoadError::new_str("Asset was not pre-loaded!"))?;
        return Ok(data);
    }
    // Load a cached asset
    pub fn load_cached(&self, name: &str) -> Result<&CachedAsset, AssetLoadError> {

    }
    // Unload asset metadata (if possible)
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