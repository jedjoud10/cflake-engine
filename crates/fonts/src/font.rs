use rendering::{Texture2D, TextureDimensionType};
use resources::{LoadableResource, Resource};

use crate::FontChar;

// A simple font containing the characters
pub struct Font {
    pub name: String,
    pub atlas_dimensions: veclib::Vector2<u16>,
    pub texture_pixels: Vec<u8>,
    pub texture: Option<Texture2D>,
    pub chars: Vec<FontChar>,
}

impl Font {
    // Get a specific character from this font using it's ASCII code
    pub fn get_char(&self, ascii_code: u8) -> &FontChar {
        // The offset of the first ASCII character in the font
        const ASCII_FIRST_CHAR_OFFSET: u8 = 33;
        let char = self.chars.get((ascii_code - ASCII_FIRST_CHAR_OFFSET) as usize).unwrap();
        return char;
    }
    // Create the actual texture from the raw pixel bitmap data we have 
    pub fn create_texture(&mut self) {
        match self.texture {
            None => { 
                // Create the texture and set it's parameters
                let texture = Texture2D::new().set_dimensions(self.atlas_dimensions.x, self.atlas_dimensions.y)
                    .set_filter(rendering::TextureFilter::Linear)
                    .set_idf(gl::RED, gl::R16, gl::UNSIGNED_BYTE);
                // Create the texture data from the bitmap pixels
                let texture = texture.generate_texture(self.texture_pixels.clone());
                self.texture = Some(texture);
            },
            _ => { /* The texture already exists */ }
        }        
    }
    // Create a font with empty paramaters and without any texture or chars
    pub fn new() -> Self {
        Self {
            name: String::new(),
            atlas_dimensions: veclib::Vector2::ZERO,
            texture_pixels: Vec::new(),
            texture: None,
            chars: Vec::new(),
        }
    }
    // Set the font parameters for this text
    pub fn set_font_parameter(&self) {}
}

// The font is loadable
impl LoadableResource for Font {
    fn from_resource(self, resource: &resources::Resource) -> Option<Self>
    where
        Self: Sized,
    {
        match resource {
            Resource::Font(font, name) => {
                // Load the chars
                let chars = font.chars.iter().map(|x| FontChar { id: x.id, min: x.min, max: x.max }).collect::<Vec<FontChar>>();
                let mut output = Self {
                    name: name.clone(),
                    atlas_dimensions: font.dimensions.into(),
                    texture_pixels: font.texture_pixels.clone(),
                    texture: None,
                    chars: chars,
                };
                // Create the OpenGL texture after the atlas was created
                output.create_texture();
                return Some(output);
            }
            _ => None,
        }
    }
}
