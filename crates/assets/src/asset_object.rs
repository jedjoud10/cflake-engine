use std::rc::Rc;

use crate::{Asset, AssetManager, Object};

// Asset object
pub trait AssetObject: Asset + Object  {
    // Cache and load. Load if the object was not cached it. We load it from it's asset metadata
    fn cache_load(self, local_path: &str, asset_manager: &mut AssetManager) -> Rc<Self> where Self: Sized + 'static {
        // Check if it was cached
        if asset_manager.object_cacher.cached(local_path) {
            return self.object_load_ot(local_path, &asset_manager.object_cacher).unwrap();
        } else {
            // The object was not cached, we must load it from the asset metadata
            let texture = self.load_asset(local_path, &mut asset_manager.asset_cacher).unwrap();
            // Then the object (cache it if neccessarry)
            let output = texture.object_cache_load(local_path, &mut asset_manager.object_cacher);
            output
        }
    }
}