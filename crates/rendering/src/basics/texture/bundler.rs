use super::{Texture, TextureBuilder, TextureDimensions, calculate_size_bytes};

// A texture bundler that creates a 2D texture array from a set of textures
pub struct TextureBundler;
impl TextureBundler {
    // Convert an array of CPU textures to a TextureArray
    // This will use the settings of the first texture in the array
    pub fn convert_texturearray(textures: &[Texture]) -> TextureBuilder {
        // We get the main dimensions from the first texture
        let dimensions = textures.get(0).unwrap().dimensions().as_texture2d().unwrap();
        let (width, height) = (dimensions.x, dimensions.y);

        // Load the bytes
        let mut bytes: Vec<u8> = Vec::with_capacity(calculate_size_bytes(textures[0]._format(), textures[0].count_texels()));
        for texture in textures {
            // Check if we have the same settings
            if texture.dimensions().as_texture2d().unwrap().x != width || texture.dimensions().as_texture2d().unwrap().y != height {
                panic!();
            }
            bytes.extend(texture.bytes().iter());
        }
        TextureBuilder::default()
            ._format(textures[0]._format())
            ._type(textures[0]._type())
            .bytes(bytes)
            .custom_params(textures[0].custom_params())
            .dimensions(TextureDimensions::Texture2dArray(veclib::Vector3::new(width, height, textures.len() as u16)))
            .filter(textures[0].filter())
            .mipmaps(textures[0].mipmaps())
            .wrap_mode(textures[0].wrap_mode())
    }
}