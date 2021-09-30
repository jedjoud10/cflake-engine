use std::collections::HashMap;

use crate::Font;

// The font manager
#[derive(Default)]
pub struct FontManager {
    pub fonts: HashMap<String, Font>,
}

impl FontManager {
    // Get a font using it's name
    pub fn get_font(&self, name: &str) -> &Font {
        return self.fonts.get(&name.to_string()).unwrap();
    }
    // Add a specific font to the font manager
    pub fn add_font(&mut self, font: Font) {
        // Get the font name
        let font_name = font.name.clone();
        // Check if we already don't have a font with the same name
        self.fonts.entry(font_name.clone()).or_insert(font);
        if self.fonts.len() == 1 {
            println!("Add font: '{}' as default font", font_name);
        };
    }
}
