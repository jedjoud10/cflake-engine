use std::{ffi::OsStr, path::{Path, PathBuf}, time::Instant};
use crate::loader::{AssetLoader, Meta, CompoundMeta};

// An asset that will be loaded from a single unique file
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: Sized {
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by this asset
    fn extensions() -> &'static [&'static str];

    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize(bytes: &[u8], args: Self::Args, meta: Meta) -> Self;
}

// A compound asset takes multiple different assets to construct a bigger asset
pub trait CompoundAsset<'args>
where
    Self: Sized,
{
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by the paths that we will use to load the asset
    fn extensions() -> &'static [&'static str];

    // Given the asset loader, we shall deserialize this compound asset using a compound context loader
    fn deserialize<'loader>(tuples: &[&[u8]], args: Self::Args, meta: CompoundMeta) -> Self;
}