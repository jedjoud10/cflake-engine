use std::{collections::HashMap, fs::Metadata, rc::Rc};
use crate::{Asset, AssetLoadError, AssetLoadType, AssetMetadata, AssetMetadataLoadError, CachedObject};

// Asset manager
pub struct AssetManager {
    // The cached asset metadata
    pub cached_metadata: HashMap<String, AssetMetadata>,
    // The cached objects
    pub cached_objects: HashMap<String, CachedObject>
}

impl AssetManager {
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
// Caching
impl AssetManager {
    // Cache a specific struct that implements the Asset trait
    pub fn cache<T: 'static + Asset>(&mut self, object_name: &str, obj: T) -> Result<Rc<T>, AssetLoadError> {
        if !self.cached(object_name) {
            // Cached asset
            let string_name = object_name.clone().to_string();
            let rc = Rc::new(obj);
            let cloned: Rc<T> = Rc::clone(&rc);
            let cached_asset = CachedObject { 
                cache_name: string_name.clone(),
                object: rc
            };
            // Only cache when the object isn't cached yet
            self.cached_objects.insert(string_name, cached_asset);
            Ok(cloned)
        } else {
            // Asset was already cached
            Err(AssetLoadError::new_str("Asset was already cached!"))
        }
    }
    // Load a cached object
    pub fn load_cached(&self, cache_name: &str) -> Result<&CachedObject, AssetLoadError> {
        let obj = self.cached_objects.get(cache_name).ok_or(AssetLoadError::new_str("Could not load cached asset!"))?;
        return Ok(obj);
    }
    // Check if an object is already cached
    pub fn cached(&self, object_name: &str) -> bool { return self.cached_objects.contains_key(object_name) }
}