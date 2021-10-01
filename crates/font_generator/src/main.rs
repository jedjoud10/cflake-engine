use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

use byteorder::{LittleEndian, WriteBytesExt};
use fonts::FontChar;
use image::GenericImageView;

// Pack the font
fn main() {
    // Get the texture atlas and BMP Font generator .fnt file
    let args: Vec<String> = env::args().collect();
    // Get the texture atlas file path
    let index = args.iter().position(|x| *x == "-t".to_string()).unwrap() + 1;
    let texture_atlas_path = args[index].clone();
    // Get the config file path
    let index = args.iter().position(|x| *x == "-c".to_string()).unwrap() + 1;
    let config_file_path = args[index].clone();
    let config_file = OpenOptions::new().read(true).open(config_file_path.clone()).unwrap();
    let reader = BufReader::new(config_file);
    // Get the output file path
    let index = args.iter().position(|x| *x == "-o".to_string()).unwrap() + 1;
    let output_file_path = args[index].clone();
    println!("Texture atlas file: {}", texture_atlas_path);
    println!("Config file: {}", config_file_path);
    println!("Output file: {}", output_file_path);

    // Get the font texture atlas image
    let texture = image::open(texture_atlas_path).unwrap();
    let dimension = texture.dimensions();
    let dimension = (dimension.0 as u16, dimension.1 as u16);
    // Get the writer
    let output_file = OpenOptions::new().create(true).truncate(true).write(true).open(output_file_path).unwrap();
    let mut output_file_writer = BufWriter::new(output_file);

    // Write the width and height
    output_file_writer.write_u16::<LittleEndian>(dimension.0).unwrap();
    output_file_writer.write_u16::<LittleEndian>(dimension.1).unwrap();

    // Read the config file data
    let mut font_chars: Vec<FontChar> = Vec::new();
    let lines = reader.lines().map(|x| x.unwrap()).collect::<Vec<String>>();
    for line in lines {
        // Get the ASCII character ID
        let split_line = line.split(" ").filter(|x| !x.is_empty()).collect::<Vec<&str>>();
        // Check if this a char line
        if split_line[0] == "char" {
            // Get the ID
            let id = split_line[1].split("id=").nth(1).unwrap().parse::<u8>().unwrap();
            // Get the min and max
            let x = split_line[2].split("x=").nth(1).unwrap().parse::<u16>().unwrap();
            let y = split_line[3].split("y=").nth(1).unwrap().parse::<u16>().unwrap();
            let width = split_line[4].split("width=").nth(1).unwrap().parse::<u16>().unwrap();
            let height = split_line[5].split("height=").nth(1).unwrap().parse::<u16>().unwrap();

            // Create the min and max from the x,y and width,height
            let min = veclib::Vector2::<u16>::new(x, y);
            let max = veclib::Vector2::<u16>::new(x + width, y + height);
            println!("{} {:?} {:?}", id, min, max);
            let font_char = FontChar { id, min, max };
            // Yes
            font_chars.push(font_char);
        }
    }

    // The threshold to detect lit pixels
    const THRESHOLD: u8 = 128;
    // Turn each pixel into a single bit first of all
    let bit_pixels: Vec<(u32, u32, bool)> = texture.pixels().map(|x| (x.0, x.1, x.2[0] > THRESHOLD)).collect();
    // Get the signed SDF for each pixel
    let mut pixels_to_write: Vec<u8> = Vec::new();
    // Create multiple vectors for each character, that way we don't have to find the sdf of the whole texture, but only of each "sub-texture" for each character
    let mut sub_textures: Vec<Vec<(u32, u32, bool)>> = Vec::new();

    for font_char in font_chars.iter() {
        // Get the sub-texture
        let sub_texture = bit_pixels.iter().filter_map(|x| {
                let valid = (x.0 as u16) > font_char.min.x && (x.1 as u16) > font_char.min.y && (x.0 as u16) < font_char.max.x && (x.1 as u16) < font_char.max.y;
                if valid {
                    Some((x.0.clone(), x.1.clone(), x.2.clone()))
                } else {
                    None
                }
            }
        ).collect::<Vec<(u32, u32, bool)>>();
        
        // Get the SDF now
        for pixel in sub_texture.iter() {
            let coords = veclib::Vector2::<f32>::new(pixel.0 as f32, pixel.1 as f32);
            let pixel_color = if !pixel.2 {
                // Keep the best distance
                let mut best_distance: f32 = f32::MAX;
                // Get the distance to lit pixels
                for sdf_pixel in sub_texture.iter() {
                    if sdf_pixel.2 {
                        let sdf_coords = veclib::Vector2::<f32>::new(pixel.0 as f32, pixel.1 as f32);
                        best_distance = best_distance.min(coords.distance(sdf_coords));
                    }
                }
                ((best_distance / 20.0).clamp(0.0, 20.0) * 12.75) as u8
            } else {
                // This pixel is already lit
                0
            };
            // Just in case
            let max = dimension.0 as u32 * dimension.1 as u32;
            if (pixels_to_write.len() as u32) < max {
                pixels_to_write.push(pixel_color);
            }
        }        
        println!("Finished creating the SDF for the character {}", font_char.id);
    }
    println!("Final pixels to write length {}", pixels_to_write.len())
;
    // Write each new pixel
    for pixel in pixels_to_write {
        output_file_writer.write_u8(pixel).unwrap();
    }

    // Write the number of characters
    output_file_writer.write_u8(font_chars.len() as u8).unwrap();

    // Write the character configs
    for font_char in font_chars.iter() {
        // Write the ID first
        output_file_writer.write_u8(font_char.id).unwrap();
        // Then you can write the min-max values
        output_file_writer.write_u16::<LittleEndian>(font_char.min.x).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.min.y).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.max.x).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.max.y).unwrap();
    }
}
