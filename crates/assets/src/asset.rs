use std::{ffi::OsStr, path::Path};

use crate::{
    loader::{AssetBytes, AssetLoader, CachedBytes},
};
// A single asset that will have access to the loader to be able to create Self
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>
where
    Self: Sized,
{
    // Extra data that can be used to construct the object
    type Args: 'args;

    // The extensions supported by this asset
    fn extensions() -> &'static [&'static str];

    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize<'loader>(bytes: AssetBytes, args: Self::Args) -> Self;
}
