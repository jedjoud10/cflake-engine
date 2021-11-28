// Some asset commands
pub mod assetc {
    pub use crate::globals::asset_cacher;
    use crate::{Asset, AssetLoadError, AssetType};
    // Load an asset
    pub fn load<T: Asset>(path: &str, obj: T) -> Result<T, AssetLoadError> {
        // Load the metadata first
        let assetcacher = asset_cacher();
        let md = assetcacher
            .cached_metadata
            .get(path)
            .ok_or(AssetLoadError::new(format!("Could not load asset '{}'!", path)))?;
        obj.load_medadata(md).ok_or(AssetLoadError::new(format!("Could not load metadata for asset '{}'!", path)))
    }
    // Load an asset (By creating a default version of it)
    pub fn dload<T: Asset + Default>(path: &str) -> Result<T, AssetLoadError> {
        load(path, T::default())
    }
    // Load an asset as UTF8 text
    pub fn load_text(path: &str) -> Result<String, AssetLoadError> {
        // Load the metadata first
        let assetcacher = asset_cacher();
        let md = assetcacher
            .cached_metadata
            .get(path)
            .ok_or(AssetLoadError::new(format!("Could not load asset '{}'!", path)))?;
        // Pls don't deadlock again
        let output = match &md.asset_type {
            // This asset is a text asset
            AssetType::Text => {
                let text = String::from_utf8(md.bytes.clone()).ok().unwrap();
                text
            }
            _ => panic!(),
        };
        Ok(output)
    }
}
// Some caching commands
pub mod cachec {
    use std::sync::Arc;

    pub use crate::globals::object_cacher;
    use crate::Asset;
    use crate::CachedObject;
    use crate::Object;
    use crate::ObjectLoadError;

    // Cache a specific Object
    pub fn cache<T: 'static + Object + Send + Sync>(object_name: &str, obj: T) -> Result<CachedObject<T>, ObjectLoadError> {
        if !cached(object_name) {
            let mut cacher = object_cacher();
            // We cache the asset
            let string_name = object_name.to_string();
            let arc = Arc::new(obj);
            // Only cache when the object isn't cached yet
            cacher.cached_objects.insert(string_name, arc.clone());
            let cached_object = CachedObject { arc };
            Ok(cached_object)
        } else {
            // Asset was already cached
            Err(ObjectLoadError::new_str("Asset was already cached!"))
        }
    }
    // Load a specific Object
    pub fn load<T: 'static + Object + Send + Sync>(cache_name: &str) -> Result<CachedObject<T>, ObjectLoadError> {
        let cacher = object_cacher();
        let obj = cacher
            .cached_objects
            .get(cache_name)
            .ok_or(ObjectLoadError::new(format!("Could not load cached object {}!", cache_name)))?;
        let arc = Arc::downcast::<T>(obj.clone()).unwrap();
        let cached_object = CachedObject { arc };
        return Ok(cached_object);
    }
    // Cache once, load endlessly
    pub fn cache_l<T: 'static + Object + Send + Sync>(object_name: &str, obj: T) -> Result<CachedObject<T>, ObjectLoadError> {
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
        let cached = cacher.cached_objects.contains_key(object_name);
        cached
    }

    // Cache once (by loading the asset), load endlessly
    pub fn acache_l<T: 'static + Object + Asset + Send + Sync>(object_name: &str, obj: T) -> Result<CachedObject<T>, ObjectLoadError> {
        if cached(object_name) {
            // Cached asset
            Ok(load(object_name).unwrap())
        } else {
            // Load from the asset, then cache it
            let asset = crate::assetc::load(object_name, obj).map_err(|x| ObjectLoadError::new(x.details))?;
            cache(object_name, asset)
        }
    }
    // Cache once (by loading the asset), load endlessly (Returns a clone of the object)
    pub fn acache_lc<T: 'static + Object + Asset + Send + Sync + Clone>(object_name: &str, obj: T) -> Result<T, ObjectLoadError> {
        Ok(acache_l(object_name, obj)?.arc.as_ref().clone())
    }
}
