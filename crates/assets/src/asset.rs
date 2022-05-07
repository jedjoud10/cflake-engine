use crate::{metadata::AssetMetadata, loader::AssetLoader};
// A single asset, that can be loaded directly from raw bytes
// Each asset has some extra data that can be used to construct the object
pub trait Asset {
    // Extra data that can be used to construct the object
    type OptArgs;

    // Check if the extension is valid
    fn is_valid(meta: AssetMetadata) -> bool;

    // Deserialize an asset, assuming that the given bytes are already in the valid format
    unsafe fn deserialize(bytes: &[u8], args: &Self::OptArgs) -> Self;

    // Load an asset by reading the asset loader's bytes and using explicity opt args
    fn try_load_with(loader: &AssetLoader, path: &str, args: &Self::OptArgs) -> Option<Self> where Self: Sized {
        let (meta, bytes) = loader.load_with::<Self>(path)?;
        Self::is_valid(meta).then(|| unsafe { Self::deserialize(&*bytes, args) })
    }

    // Load an asset using default opt args
    fn try_load(loader: &AssetLoader, path: &str, args: &Self::OptArgs) -> Option<Self> where Self: Sized, Self::OptArgs: Default {
        Self::try_load_with(loader, path, &Default::default())
    }
}
