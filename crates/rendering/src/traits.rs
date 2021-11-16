use std::rc::Rc;

use assets::{Asset, AssetManager, AssetMetadata};
use rendering_base::{error::RenderingError, texture::{TextureLoadOptions, TextureType}};

// A Render Asset Object
pub trait RenderAssetObject {
    // Load a texture from scratch
    fn load_medadata(self, data: &AssetMetadata) -> Option<Self> where Self: Sized;
}

// Now for the indual traits
pub trait TextureT {
    // Read bytes
    fn read_bytes(metadata: &AssetMetadata) -> (Vec<u8>, u16, u16);
    // Update size
    fn update_size(&mut self, ttype: TextureType);
    // Create texture array
    fn create_texturearray(load_options: Option<TextureLoadOptions>, texture_paths: Vec<&str>, asset_manager: &mut AssetManager, width: u16, height: u16) -> Self;
    // Generate texture
    fn generate_texture(self, bytes: Vec<u8>) -> Result<Self, RenderingError> where Self: Sized;
    // Update data
    fn update_data(&mut self, bytes: Vec<u8>);
}