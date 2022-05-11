use crate::{loader::AssetLoader, metadata::AssetMetadata};

// An asset is a specific type of resource that we can fetch from the asset manager
// Assets can be used for general purpose caching
pub trait Asset<'args> {

}

// An asset file is an asset that is represented by a singular file, like a texture or a sound effect
pub trait AssetFile<'args>: Asset<'args> {
    // Check if the metadat for a specific asset is valid
    fn is_valid(meta: AssetMetadata) -> bool;

    // Deserialize an asset, assuming that the given bytes are already in the valid format
    unsafe fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Option<Self>
    where
        Self: Sized;
}

pub trait Asset<'args> {
    // Extra data that can be used to construct the object
    type OptArgs: 'args;





    // Load an asset by reading the asset loader's bytes and using explicity opt args
    fn try_load_with<'l>(loader: &AssetLoader, path: &str, args: Self::OptArgs) -> Option<Self>
    where
        Self: Sized,
    {
        let (meta, bytes) = loader.load_with::<Self>(path)?;
        Self::is_valid(meta).then(|| unsafe { Self::deserialize(&*bytes, args) }).flatten()
    }

    // Load an asset using default opt args
    fn try_load<'l>(loader: &'l AssetLoader, path: &str) -> Option<Self>
    where
        Self: Sized,
        Self::OptArgs: Default,
    {
        Self::try_load_with(loader, path, Default::default())
    }
}
