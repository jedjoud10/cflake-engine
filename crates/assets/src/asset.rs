use std::{ffi::OsStr, path::{Path, PathBuf}, time::Instant};
use crate::loader::{AssetLoader, CachedSlice};

// An asset that will be loaded from a single unique file
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: Sized {
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by this asset
    fn extensions() -> &'static [&'static str];

    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize(bytes: CachedSlice, args: Self::Args) -> Self;
}