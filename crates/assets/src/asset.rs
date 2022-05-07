use crate::metadata::AssetMetadata;
// A single asset, that can be loaded directly from raw bytes
// Each asset has some extra data that can be used to construct the object
pub trait Asset {
    // Extra data that can be used to construct the object
    type OptArgs;

    // The file extension that we MUST use for loading this specific asset
    const EXTENSION: &'static str;

    // Deserialize the asset with everything that is given from the asset loader
    fn try_deserialize(meta: &AssetMetadata, bytes: &[u8], args: Self::OptArgs) -> Option<Self>
    where
        Self: Sized {
        // Make sure the extension matches up
        (meta.extension.as_os_str().to_str()? == Self::EXTENSION).then(|| Self::deserialize(bytes, args))
    }


    // Deserialize the byte data and extra data into the object
    fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Self
    where
        Self: Sized;
}
