use std::{ffi::OsStr, path::Path};

use crate::{loader::{AssetLoader, LoadingContext}, LoadError};
// A single asset that will have access to the loader to be able to create Self
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: where Self: Sized {
    // Extra data that can be used to construct the object
    type OptArgs: 'args;

    // Check if a file path extension can be used to load this asset
    fn is_extension_valid(extension: &str) -> bool;
    
    // Load the asset from the loader using a specific loading context and arguments
    fn load(loader: &mut AssetLoader, path: &str, args: Self::OptArgs, _ctx: LoadingContext) -> Self;

    // Deserialize an asset, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Self;
}