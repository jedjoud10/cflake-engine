use crate::{main, Asset, Object};
use std::sync::Arc;

// Asset object
pub trait AssetObject: Asset + Object {
    // Cache and load. Load if the object was not cached it. We load it from it's asset metadata
    fn cache_load(self, local_path: &str) -> CachedObject<Self>
    where
        Self: Sized + 'static,
    {
        todo!()
        /*
        // Check if it was cached
        if alocc::object_cacher().cached(local_path) {
            return self.object_load_ot(local_path).unwrap();
        } else {
            // The object was not cached, we must load it from the asset metadata
            let texture = self.load_asset(local_path).unwrap();
            // Then the object (cache it if neccessarry)
            let output = texture.object_cache_load(local_path);
            output
        }
        */
    }
}

// Cached asset object
pub struct CachedObject<T>
where
    T: Send + Sync,
{
    pub arc: Arc<T>,
}
