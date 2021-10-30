use std::{any::Any, rc::Rc};

use crate::AssetManager;

// For how long will this asset be alive?
pub enum AssetLoadType {
    Static, // You can only load it, you can't unload it
    Dynamic, // You can load it, and you can also unload it
    CustomCached, // Dispose of the bytes data, since the asset is customly cached
}
// Some data
pub struct AssetMetadata {
    // Bytes
    pub bytes: Vec<u8>,
    // Doodoo water
    pub load_type: AssetLoadType
}
impl AssetMetadata {
    // Turn the bytes into a UTF8 string
    pub fn read_string(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }
}

// A cached asset
pub struct CachedObject {
    pub cache_name: String,
    pub object: Rc<dyn Any>
}

impl CachedObject {
    // Cast this cached asset to a specific struct
    pub fn cast<T: Sized + 'static>(&self) -> &T {
        let r = &self.object;
        let t = r.downcast_ref::<T>().unwrap();
        return t;
    }
}

// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset from metadata
    fn load(data: &AssetMetadata) -> Self where Self: Sized;
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_t(self, data: &AssetMetadata) -> Self where Self: Sized {
        panic!()
    }    
    // Load this asset as a cached asset, but also cache it if it was never loaded
    fn cl_object(self, object_name: &str, asset_manager: &mut AssetManager) -> Rc<Self> where Self: Sized + 'static {
        // Check if it was cached or not
        if asset_manager.cached(object_name) {
            // This object is cached
            let object = asset_manager.load_cached(object_name).unwrap();
            let any = &object.object;
            let t = any;
            let borrow = t.as_ref().unwrap(); 
            // Put it back into an Rc
            let rc_object = Rc::clone(borrow);
            rc_object
        } else {
            // This object was not cached, cache it
            let rc_object = asset_manager.cache(object_name, self).unwrap();
            rc_object
        }
    }
    // Any
    fn as_any(&self) -> &dyn Any;
}