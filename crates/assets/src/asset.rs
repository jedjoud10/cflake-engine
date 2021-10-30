use std::rc::Rc;

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
pub struct CachedAsset {
    pub local_path: String,
    pub object: Rc<dyn Asset>
}

// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset from some bytes with some sprinkles of metadata
    fn load(data: &AssetMetadata) -> Self where Self: Sized;
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_t(self, data: &AssetMetadata) -> Self where Self: Sized {
        panic!()
    }    
}