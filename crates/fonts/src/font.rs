use rendering::Texture;
use crate::FontChar;

// A simple font containing the characters
pub struct Font {
    pub texture_atlas_id: u16,
    pub chars: Vec<FontChar>
}

impl Font {
    // Get a specific character from this font using it's ASCII code
    pub fn get_char(&self, ascii_code: u8) -> &FontChar {
        // The offset of the first ASCII character in the font 
        const ASCII_FIRST_CHAR_OFFSET: u8 = 33;
        let char = self.chars.get((ascii_code - ASCII_FIRST_CHAR_OFFSET) as usize).unwrap();
        return char;
    }
    // Load a texture into this font
    pub fn load_texture(&mut self, texture_atlas_id: u16) {
        self.texture_atlas_id = texture_atlas_id
    }
}