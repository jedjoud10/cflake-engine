use resources::{LoadableResource, Resource};

use crate::FontChar;

// A simple font containing the characters
pub struct Font {
    pub name: String,
    pub atlas_dimensions: veclib::Vector2<u32>,
    pub texture_pixels: Vec<u8>,
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
    // Create a font with empty paramaters and without any texture or chars
    pub fn new() -> Self {
        Self {
            name: String::new(),
            atlas_dimensions: veclib::Vector2::ZERO,
            texture_pixels: Vec::new(),
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
                let output = Self {
                    name: name.clone(),
                    atlas_dimensions: font.dimensions,
                    texture_pixels: font.texture_pixels.clone(),
                    chars: chars,
                };
                return Some(output);
            }
            _ => None,
        }
    }
}
