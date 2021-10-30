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
// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset from some bytes with some sprinkles of metadata
    fn load(data: AssetMetadata) -> Self where Self: Sized;
}