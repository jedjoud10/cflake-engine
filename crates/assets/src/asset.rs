use crate::metadata::AssetMetadata;
// A single asset, that can be loaded directly from raw bytes
pub trait Asset {
    // Load this asset, but only if we already have some data initalized in the struct
    fn deserialize(self, meta: &AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;
}
