use crate::metadata::AssetMetadata;
// A single asset, that can be loaded directly from raw bytes
pub trait Asset {
    // Deserialize the raw bytes data into the proper asset
    fn deserialize(meta: &AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;
}
