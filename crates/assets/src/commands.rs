// Some asset commands
pub mod assetc {
    pub use crate::globals::asset_cacher;
    use crate::{Asset, AssetLoadError, AssetType};
    // Load an asset
    pub fn load<T: Asset>(path: &str, obj: T) -> Result<T, AssetLoadError> {
        // Load the metadata first
        let assetcacher = asset_cacher();
        let md = assetcacher
            .cached_metadata
            .get(path)
            .ok_or(AssetLoadError::new(format!("Could not load asset '{}'!", path)))?;
        obj.load_medadata(md).ok_or(AssetLoadError::new(format!("Could not load metadata for asset '{}'!", path)))
    }
    // Load an asset (By creating a default version of it)
    pub fn dload<T: Asset + Default>(path: &str) -> Result<T, AssetLoadError> {
        load(path, T::default())
    }
    // Load an asset as UTF8 text
    pub fn load_text(path: &str) -> Result<String, AssetLoadError> {
        // Load the metadata first
        let assetcacher = asset_cacher();
        let md = assetcacher
            .cached_metadata
            .get(path)
            .ok_or(AssetLoadError::new(format!("Could not load asset '{}'!", path)))?;
        // Pls don't deadlock again
        let output = match &md.asset_type {
            // This asset is a text asset
            AssetType::Text => {
                let text = String::from_utf8(md.bytes.clone()).ok().unwrap();
                text
            }
            _ => panic!(),
        };
        Ok(output)
    }
}