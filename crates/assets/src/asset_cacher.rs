use std::collections::HashMap;

use crate::{AssetMetadataLoadError, alocc};

// Caches the embeded bytes into an array basically
#[derive(Default)]
pub struct AssetCacher {
    // The cached metadata
    pub cached_metadata: HashMap<String, AssetMetadata>,
}

impl AssetCacher {
    // Guess the asset type of a specific asset using it's name
    fn guess_asset_type(name: &str) -> AssetType {
        let first_dot_index = name.split("").position(|c| c == ".").unwrap();
        let extension = name.split_at(first_dot_index).1;
        match extension {
            "vrsh.glsl" => AssetType::VertSubshader,
            "frsh.glsl" => AssetType::FragSubshader,
            "mdl3d" => AssetType::Model,
            "cmpt.glsl" => AssetType::ComputeSubshader,
            "func.glsl" => AssetType::Text,
            "png" => AssetType::Texture,
            "font" => AssetType::Font,
            _ => {
                /* Nothing */
                panic!()
            }
        }
    }
    // Pre-load some asset metadata
    pub fn pre_load(&mut self, name: &str, bytes: &[u8]) -> Result<(), AssetMetadataLoadError> {
        let name = name.split("resources\\").last().unwrap();
        let data = AssetMetadata {
            bytes: bytes.to_vec(),
            load_type: AssetLoadType::Dynamic,
            asset_type: Self::guess_asset_type(name),
            name: name.to_string(),
        };
        self.cached_metadata.insert(name.to_string(), data);
        Ok(())
    }
    // Load asset metadata
    pub fn load_md(&self, name: &str) -> Result<&AssetMetadata, AssetMetadataLoadError> {
        // Load
        let data = self
            .cached_metadata
            .get(name)
            .ok_or(AssetMetadataLoadError::new(format!("Asset '{}' was not pre-loaded!", name)))?;
        return Ok(data);
    }
    // Load a text file from the asset cacher
    pub fn load_text(&self, name: &str) -> Result<String, AssetMetadataLoadError> {
        let md = self.load_md(name)?;
        match &md.asset_type {
            // This asset is a text asset
            AssetType::Text => {
                let text = String::from_utf8(md.bytes.clone()).ok().unwrap();
                return Ok(text);
            }
            _ => {
                panic!()
            }
        }
    }
    // Unload asset metadata (if possible)
    pub fn unload(&mut self, name: &str) -> Result<AssetMetadata, AssetMetadataLoadError> {
        // Check the load type
        let _type = self
            .cached_metadata
            .get(name)
            .ok_or(AssetMetadataLoadError::new_str("Asset is not loaded in the first place!"))?;
        match &_type.load_type {
            AssetLoadType::Dynamic => {
                // Unload
                return Ok(self.cached_metadata.remove(name).unwrap());
            }
            _ => { /* Nothing */ }
        }
        Err(AssetMetadataLoadError::new_str("No"))
    }
}

// For how long will this asset be alive?
pub enum AssetLoadType {
    Static,       // You can only load it, you can't unload it
    Dynamic,      // You can load it, and you can also unload it
    CustomCached, // Dispose of the bytes data, since the asset is customly cached
}
// Asset type
pub enum AssetType {
    VertSubshader,
    FragSubshader,
    ComputeSubshader,
    Text,
    Texture,
    Model,
    Sound,
    Font,
}
// Some data
pub struct AssetMetadata {
    // Bytes
    pub bytes: Vec<u8>,
    // Doodoo water
    pub load_type: AssetLoadType,
    pub asset_type: AssetType,
    pub name: String,
}
impl AssetMetadata {
    // Turn the bytes into a UTF8 string
    pub fn read_string(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }
}
// A single asset, that can be loaded directly from raw bytes bundled in the .dll
pub trait Asset {
    // Load this asset, but only if we already have some data initalized in the struct
    fn load_medadata(self, data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized;
    // Combination of the two
    fn load_asset(self, local_path: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let ac = alocc::asset_cacher();
        let s = ac.load_md(local_path).ok()?;
        self.load_medadata(s)
    }
}
