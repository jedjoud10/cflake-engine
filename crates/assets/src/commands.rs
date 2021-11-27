// Some asset commands
pub mod assetc {
    use crate::Asset;

    pub fn load<T: Asset>(path: &str) -> Option<T> {
        None
    }
}
// Some caching commands
pub mod cachec {
    use std::sync::Arc;

    use crate::Asset;
    use crate::Object;
    use crate::ObjectLoadError;
    use crate::main::*;

    // Cache a specific Object
    pub fn cache<T: 'static + Object + Send + Sync>(object_name: &str, obj: T) -> Result<Arc<T>, ObjectLoadError> {
        let mut cacher = object_cacher();
        if cached(object_name) {
            // We cache the asset
            let string_name = object_name.to_string();
            let arc = Arc::new(obj);
            // Only cache when the object isn't cached yet
            cacher.cached_objects.insert(string_name, arc.clone());
            Ok(arc)
        } else {
            // Asset was already cached
            Err(ObjectLoadError::new_str("Asset was already cached!"))
        }
    }
    // Load a specific Object
    pub fn load<T: 'static + Object + Send + Sync>(cache_name: &str) -> Result<Arc<T>, ObjectLoadError> {
        let cacher = object_cacher();
        let obj = cacher.cached_objects.get(cache_name).ok_or(ObjectLoadError::new_str("Could not load cached asset!"))?;
        let obj = Arc::downcast::<T>(obj.clone()).unwrap();
        return Ok(obj);
    }
    // Cache once, load endlessly
    pub fn cache_once_load<T: 'static + Object + Send + Sync>(object_name: &str, obj: T) -> Result<Arc<T>, ObjectLoadError> {
        if cached(object_name) {
            // Cached asset
            Ok(load(object_name).unwrap())
        } else {
            // Cache it
            let x = cache(object_name, obj)?;
            Ok(x)
        }
    }
    // Check if an Object is cached
    pub fn cached(object_name: &str) -> bool {
        let cacher = object_cacher();
        return cacher.cached_objects.contains_key(object_name);
    }

    // Cache once (by loading the asset), load endlessly
    pub fn cache_once_load_asset<T: 'static + Object + Asset + Send + Sync>(object_name: &str, obj: T) -> Result<Arc<T>, ObjectLoadError> {
        if cached(object_name) {
            // Cached asset
            Ok(load(object_name).unwrap())
        } else {
            // Load from the asset, then cache it
            let asset = crate::assetc::load(object_name).unwrap();
            let output = cache(object_name, asset);
            output
        }
    }
}