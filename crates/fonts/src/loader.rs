use std::process::Command;
use crate::FontChar;

// Create a font by getting the SDF image from Hiero
// NOTE: Only supports monospaced fonts because I a m lazy
pub struct FontGenerator {
    pub chars: Vec<FontChar>,
}

impl FontGenerator {
    // Getting the SDF image and the layout file
    pub fn get_font_characters(file_path: &str, config_path: &str) {
        // Load the file
        
    }
}