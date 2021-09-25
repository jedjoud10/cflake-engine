use std::{fs::OpenOptions, io::BufReader};

use image::GenericImageView;

pub struct FontPacker {
}

// A packed font 

// Creates a packed SDF font file that can be loaded later
impl FontPacker {
    // Generate the custom font resource file by taking in the font texture atlas and the config file
    pub fn generate_font(&self, texture_atlas_path: &str, config_file_path: &str) {
        
    }
}