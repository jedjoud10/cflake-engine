use std::{env, fs::{File, OpenOptions}, io::{BufRead, BufReader, BufWriter}, path::Path};

use byteorder::{LittleEndian, WriteBytesExt};
use fonts::FontChar;
use image::GenericImageView;

// Pack the font
fn main() {
    // Get the texture atlas and BMP Font generator .fnt file
    let args: Vec<String> = env::args().collect();
    // Get the texture atlas file path
    let index = args.iter().position(|x| *x == "-t".to_string()).unwrap()+1;
    let texture_atlas_path = args[index].clone();
    // Get the config file path
    let index = args.iter().position(|x| *x == "-c".to_string()).unwrap()+1;
    let config_file_path = args[index].clone();
    let config_file = OpenOptions::new().read(true).open(config_file_path.clone()).unwrap();
    let reader = BufReader::new(config_file);
    // Get the output file path
    let index = args.iter().position(|x| *x == "-o".to_string()).unwrap()+1;
    let output_file_path = args[index].clone();    
    println!("Texture atlas file: {}", texture_atlas_path);
    println!("Config file: {}", config_file_path);
    println!("Output file: {}", output_file_path);
    
    // Get the font texture atlas image
    let texture = image::open(texture_atlas_path).unwrap();
    let dimension = texture.dimensions();
    
    // Get the writer
    let output_file = OpenOptions::new().create(true).truncate(true).write(true).open(output_file_path).unwrap();
    let mut output_file_writer = BufWriter::new(output_file);

    // Write the width and height
    output_file_writer.write_u32::<LittleEndian>(dimension.0).unwrap();
    output_file_writer.write_u32::<LittleEndian>(dimension.1).unwrap();

    // Write each pixel
    for pixel in texture.pixels() {
        output_file_writer.write_u8(pixel.2[0]).unwrap();
    }

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
            let x = split_line[2].split("x=").nth(1).unwrap().parse::<u32>().unwrap();
            let y = split_line[3].split("y=").nth(1).unwrap().parse::<u32>().unwrap();
            let width = split_line[4].split("width=").nth(1).unwrap().parse::<u32>().unwrap();
            let height = split_line[5].split("height=").nth(1).unwrap().parse::<u32>().unwrap();

            // Create the min and max from the x,y and width,height
            let min = veclib::Vector2::<u32>::new(x, y);
            let max = veclib::Vector2::<u32>::new(x + width, y + height);
            println!("{} {:?} {:?}", id, min, max);
        }
    }

    // Write the number of characters
    output_file_writer.write_u8(font_chars.len() as u8).unwrap();

    // Write the character configs
    for font_char in font_chars.iter() {
        // Write the ID first
        output_file_writer.write_u8(font_char.id).unwrap();
        // Then you can write the min-max values
        output_file_writer.write_u32::<LittleEndian>(font_char.min.x).unwrap();
        output_file_writer.write_u32::<LittleEndian>(font_char.min.y).unwrap();
        output_file_writer.write_u32::<LittleEndian>(font_char.max.x).unwrap();
        output_file_writer.write_u32::<LittleEndian>(font_char.max.y).unwrap();
    }
}
