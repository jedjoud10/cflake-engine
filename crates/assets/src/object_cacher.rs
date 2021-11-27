use crate::{globals, CachedObject, ObjectLoadError};
use std::{any::Any, collections::HashMap, ops::DerefMut, sync::Arc};

// The object cacher. Just holds the data about the objects
#[derive(Default)]
pub struct ObjectCacher {
    // Cached object
    pub cached_objects: HashMap<String, Arc<dyn Any + Send + Sync>>,
}
/*
    // Load this asset as a cached asset, but with preinitialized self
    fn object_load_ot(self, local_path: &str) -> Option<CachedObject<Self>>
    where
        Self: Sized + 'static,
    {
        let name = self.get_unique_object_name(local_path);
        let oc = alocc::object_cacher();
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
*/

// An object that will be cached inside the object cacher
pub trait Object
where
    Self: Sync + Send,
{
    // Get unique object
    fn get_unique_object_name(&self, local_path: &str) -> String {
        local_path.to_string()
    }    
}
