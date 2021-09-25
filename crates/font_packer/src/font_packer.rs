use std::{fs::OpenOptions, io::BufReader};

use image::GenericImageView;

pub struct FontPacker {
}

// A packed font 

// Creates a packed SDF font file that can be loaded later
impl FontPacker {
    // Getting the SDF image and the layout file
    pub fn get_font_characters(texture_atlas_file: &str, config_path: &str) ->  {
        // Load the texture atlas file
        let image = image::open(texture_atlas_file).unwrap();
        println!("{:?}", image.dimensions());
       
        // Read the config file
    }
}