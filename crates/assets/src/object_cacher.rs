use std::{any::Any, collections::HashMap, ops::DerefMut, sync::Arc};
use crate::{CachedObject, ObjectLoadError, alocc};

// The object cacher
#[derive(Default)]
pub struct ObjectCacher {
    // Cached object
    pub cached_objects: HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl ObjectCacher {
    // Cache a specific struct that implements the Asset trait
    pub fn cache<T: 'static + Object + Send + Sync>(&mut self, object_name: &str, obj: T) -> Result<Arc<T>, ObjectLoadError> {
        if !self.cached(object_name) {
            // Cached asset
            let string_name = object_name.to_string();
            let arc = Arc::new(obj);
            // Only cache when the object isn't cached yet
            self.cached_objects.insert(string_name, arc.clone());
            Ok(arc)
        } else {
            // Asset was already cached
            Err(ObjectLoadError::new_str("Asset was already cached!"))
        }
    }
    // Load a cached object
    pub fn load_cached(&self, cache_name: &str) -> Result<&Arc<dyn Any + Send + Sync>, ObjectLoadError> {
        let obj = self.cached_objects.get(cache_name).ok_or(ObjectLoadError::new_str("Could not load cached asset!"))?;
        return Ok(obj);
    }
    // Check if an object is already cached
    pub fn cached(&self, object_name: &str) -> bool {
        return self.cached_objects.contains_key(object_name);
    }
}

// An object that will be cached inside the object cacher
pub trait Object where Self: Sync + Send {
    // Get unique object
    fn get_unique_object_name(&self, local_path: &str) -> String {
        local_path.to_string()
    }
    // Only load this object knowing that it was already cached
    fn object_load_o(local_path: &str, object_cacher: &ObjectCacher) -> CachedObject<Self>
    where
        Self: Sized + 'static,
    {
        if object_cacher.cached(local_path) {
            // This object is cached
            let object = object_cacher.load_cached(local_path).unwrap();
            let arc: Arc<Self> =  Arc::downcast(object.clone()).unwrap();
            CachedObject { arc }
        } else {
            // This object was not cached, not good
            panic!()
        }
    }
    // Load this asset as a cached asset, but also cache it if it was never loaded
    fn object_cache_load(self, local_path: &str) -> CachedObject<Self>
    where
        Self: Sized + 'static,
    {
        let name = self.get_unique_object_name(local_path);
        let mut oc = alocc::object_cacher_mut();
        // Check if it was cached or not
        if oc.cached(&name) {
            // This object is cached
            let object = oc.load_cached(&name).unwrap();
            let arc = object.clone().downcast::<Self>().unwrap();
            CachedObject { arc }
        } else {
            // This object was not cached, cache it
            let arc = oc.cache(&name, self).unwrap();
            CachedObject { arc }
        }
    }
    // Load this asset as a cached asset, but with preinitialized self
    fn object_load_ot(self, local_path: &str) -> Option<CachedObject<Self>>
    where
        Self: Sized + 'static,
    {
        let name = self.get_unique_object_name(local_path);
        let oc = alocc::object_cacher_mut();
        // Check if it was cached or not
        if oc.cached(&name) {
            // This object is cached
            let object = oc.load_cached(&name).unwrap();
            let arc = object.clone().downcast::<Self>().unwrap();
            Some(CachedObject { arc })
        } else {
            None
        }
    }
}
