use std::rc::Rc;

use assets::{Asset, AssetManager, AssetMetadata};
use rendering_base::{error::RenderingError, texture::{TextureLoadOptions, TextureType}};

// A RenderAsset. Could be a texture or a model, we just need this trait to wrap around the Asset trait really
pub trait RenderAsset {
    // Load a texture from scratch
    fn asset_load(data: &AssetMetadata) -> Option<Self> where Self: Sized;
    // Load a texture that already has it's parameters set
    fn asset_load_t(self, data: &AssetMetadata) -> Option<Self> where Self: Sized;
    // Object cache load
    fn cache_load(self, local_path: &str, asset_manager: &mut AssetManager) -> Rc<Self> where Self: Sized;
}

// A render object. Any struct that is related to rendering really
pub trait RenderObject: Default {
    // New
    fn new() -> Self;
    // Finalize
    fn finalize(self) -> Self where Self: Sized;
}

// Now for the indual traits
pub trait TextureT {
    // Read bytes
    fn read_bytes(metadata: &AssetMetadata) -> (Vec<u8>, u16, u16);
    // Update size
    fn update_size(&mut self, ttype: TextureType);
    // Create texture array
    fn create_texturearray(load_options: Option<TextureLoadOptions>, texture_paths: Vec<&str>, asset_manager: &mut AssetManager, width: u16, height: u16) -> Rc<Self>;
    // Generate texture
    fn generate_texture(self, bytes: Vec<u8>) -> Result<Self, RenderingError> where Self: Sized;
    // Update data
    fn update_data(&mut self, bytes: Vec<u8>);
}