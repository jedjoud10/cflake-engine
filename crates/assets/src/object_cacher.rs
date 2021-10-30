use std::{any::Any, collections::HashMap, rc::Rc};

use crate::ObjectLoadError;

// The object cacher
pub struct ObjectCacher {
    // Cached object
    pub cached_objects: HashMap<String, Rc<dyn Any>>
}

impl ObjectCacher {
    // Cache a specific struct that implements the Asset trait
    pub fn cache<T: 'static + Object>(&mut self, object_name: &str, obj: T) -> Result<Rc<T>, ObjectLoadError> {
        if !self.cached(object_name) {
            // Cached asset
            let string_name = object_name.clone().to_string();
            let rc = Rc::new(obj);
            // Only cache when the object isn't cached yet
            self.cached_objects.insert(string_name, rc.clone());
            Ok(rc)
        } else {
            // Asset was already cached
            Err(ObjectLoadError::new_str("Asset was already cached!"))
        }
    }
    // Load a cached object
    pub fn load_cached(&self, cache_name: &str) -> Result<&Rc<dyn Any>, ObjectLoadError> {
        let obj = self.cached_objects.get(cache_name).ok_or(ObjectLoadError::new_str("Could not load cached asset!"))?;
        return Ok(obj);
    }
    // Check if an object is already cached
    pub fn cached(&self, object_name: &str) -> bool { return self.cached_objects.contains_key(object_name) }
}

// An object that will be cached inside the object cacher
pub trait Object {    
    // Get unique object 
    fn get_unique_object_name(&self, local_path: &str) -> String {
        local_path.to_string()
    }
    // Only load this object knowing that it was already cached
    fn object_load_o(local_path: &str, object_cacher: &ObjectCacher) -> Rc<Self> where Self: Sized + 'static {
        if object_cacher.cached(local_path) {
            // This object is cached
            let object = object_cacher.load_cached(local_path).unwrap();
            let any = &object.clone().downcast::<Self>().unwrap();
            // Put it back into an Rc
            let rc_object = Rc::clone(any);
            rc_object
        } else {
            // This object was not cached, not good
            panic!()
        }
    }
    // Load this asset as a cached asset, but also cache it if it was never loaded
    fn object_cache_load(self, local_path: &str, object_cacher: &mut ObjectCacher) -> Rc<Self> where Self: Sized + 'static {
        let name = self.get_unique_object_name(local_path);
        // Check if it was cached or not
        if object_cacher.cached(&name) {
            // This object is cached
            let object = object_cacher.load_cached(&name).unwrap();
            let any = &object.clone().downcast::<Self>().unwrap();
            // Put it back into an Rc
            let rc_object = Rc::clone(any);
            rc_object
        } else {
            // This object was not cached, cache it
            let rc_object = object_cacher.cache(&name, self).unwrap();
            rc_object
        }
    }
}