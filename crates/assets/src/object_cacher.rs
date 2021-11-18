use std::{any::Any, collections::HashMap, rc::Rc, sync::{Arc, Mutex}};

use crate::ObjectLoadError;

// The object cacher
#[derive(Default)]
pub struct ObjectCacher {
    // Cached object
    pub cached_objects: HashMap<String, Arc<(dyn Any + Send + Sync + 'static)>>,
}

impl ObjectCacher {
    // Cache a specific struct that implements the Asset trait
    pub fn cache<T: 'static + Object + Send + Sync + 'static>(&mut self, object_name: &str, obj: T) -> Result<Arc<T>, ObjectLoadError> {
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
    pub fn load_cached(&self, cache_name: &str) -> Result<&Arc<dyn Any + Send + Sync + 'static>, ObjectLoadError> {
        let obj = self.cached_objects.get(cache_name).ok_or(ObjectLoadError::new_str("Could not load cached asset!"))?;
        return Ok(obj);
    }
    // Check if an object is already cached
    pub fn cached(&self, object_name: &str) -> bool {
        return self.cached_objects.contains_key(object_name);
    }
}

// An object that will be cached inside the object cacher
pub trait Object: Send + Sync {
    // Get unique object
    fn get_unique_object_name(&self, local_path: &str) -> String {
        local_path.to_string()
    }
    // Only load this object knowing that it was already cached
    fn object_load_o(local_path: &str, object_cacher: &ObjectCacher) -> Arc<Self>
    where
        Self: Sized + 'static,
    {
        if object_cacher.cached(local_path) {
            // This object is cached
            let object = object_cacher.load_cached(local_path).unwrap();
            let arc_object = Arc::downcast::<Self>(object.clone()).unwrap();
            arc_object
        } else {
            // This object was not cached, not good
            panic!()
        }
    }
    // Load this asset as a cached asset, but also cache it if it was never loaded
    fn object_cache_load(self, local_path: &str, object_cacher: &mut ObjectCacher) -> Arc<Self>
    where
        Self: Sized + 'static,
    {
        let name = self.get_unique_object_name(local_path);
        // Check if it was cached or not
        if object_cacher.cached(&name) {
            // This object is cached
            let object = object_cacher.load_cached(&name).unwrap();
            let object = Arc::downcast::<Self>(object.clone()).unwrap();
            object
        } else {
            // This object was not cached, cache it
            let rc_object = object_cacher.cache(&name, self).unwrap();
            rc_object
        }
    }
    // Load this asset as a cached asset, but with preinitialized self
    fn object_load_ot(self, local_path: &str, object_cacher: &ObjectCacher) -> Option<Arc<Self>>
    where
        Self: Sized + 'static,
    {
        let name = self.get_unique_object_name(local_path);
        // Check if it was cached or not
        if object_cacher.cached(&name) {
            // This object is cached
            let object = object_cacher.load_cached(&name).unwrap();
            let arc_object = object.clone().downcast::<Self>().unwrap();
            Some(arc_object)
        } else {
            None
        }
    }
}
