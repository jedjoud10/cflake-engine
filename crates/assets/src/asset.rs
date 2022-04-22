use crate::metadata::AssetMetadata;
// A single asset, that can be loaded directly from raw bytes
// Each asset has some extra data that can be used to construct the object
pub trait Asset {
    // Extra data that can be used to construct the object
    type Input;

    // Deserialize the byte data and extra data into the object
    fn deserialize(meta: &AssetMetadata, bytes: &[u8], input: Self::Input) -> Option<Self>
    where
        Self: Sized;
}
