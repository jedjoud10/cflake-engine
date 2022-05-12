use std::{ffi::OsStr, path::Path};

use crate::{loader::{AssetLoader, Validated, CachedBytes, AssetBytes}, LoadError};
// A single asset that will have access to the loader to be able to create Self
// Each asset has some extra data that can be used to construct the object
pub trait Asset<'args>: where Self: Sized {
    // Extra data that can be used to construct the object
    type Args: 'args;

    // Check if a file path extension can be used to load this asset
    fn is_extension_valid(extension: &str) -> bool;
    
    // Deserialize asset bytes, assuming that the given bytes are already in the valid format to deserialize
    fn deserialize<'loader>(bytes: AssetBytes, args: Self::Args) -> Self;

    // Validate some specific bytes by making sure the path exists and that it is valid for this specific asset
    fn validate_bytes<'loader>(bytes: CachedBytes<'loader>, path: & str) -> Result<Validated<CachedBytes<'loader>>, LoadError> {
        // Get the file extension (if possible) from the path
        let extension = Path::new(path).extension().and_then(OsStr::to_str).ok_or(LoadError::Invalid(path.to_string()))?;

        // Check if the extension is valid, and return the validated bytes if it is
        if !Self::is_extension_valid(extension) { Err(LoadError::Invalid(path.to_string())) } else { Ok(Validated(bytes)) }
    }

    // Load an asset by reading the asset loader's bytes and using explicit opt args
    fn try_load_with<'loader>(loader: &'loader mut AssetLoader, path: &str, args: Self::Args) -> Result<Self, LoadError> {
        // Make sure the path extension is valid and that it actually exists
        let cached = loader.load(path).ok_or(LoadError::Invalid(path.to_string()));
        let validated = cached.and_then(|cached| Self::validate_bytes(cached, path))?;

        // Deserialize the asset with the new validated bytes that we have
        let asset = Self::deserialize(validated, args);
        Ok(asset)
    }

    // Load an asset using default opt args
    fn try_load<'loader>(loader: &'loader mut AssetLoader, path: &str) -> Result<Self, LoadError>
    where
        Self: Sized,
        Self::Args: Default,
    {
        Self::try_load_with(loader, path, Default::default())
    }
}