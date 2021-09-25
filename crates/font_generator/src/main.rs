use std::{env, fs::OpenOptions, io::BufWriter};

use byteorder::{LittleEndian, WriteBytesExt};
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
    // Get the output file path
    let index = args.iter().position(|x| *x == "-c".to_string()).unwrap()+1;
    let output_file_path = args[index].clone();    
    println!("Texture atlas file: {}", texture_atlas_path);
    println!("Config file: {}", config_file_path);
    println!("Output file: {}", output_file_path);
    
    // Get the font texture atlas image
    let texture = image::open(texture_atlas_path).unwrap();
    let dimension = texture.dimensions();
    
    // Get the writer
    let output_file = OpenOptions::new().write(true).open(output_file_path).unwrap();
    let mut output_file_writer = BufWriter::new(output_file);

    // Write the width and height
    output_file_writer.write_u32::<LittleEndian>(dimension.0).unwrap();
    output_file_writer.write_u32::<LittleEndian>(dimension.1).unwrap();

    // Write each pixel
    for pixel in texture.pixels() {
        let pixel_value = pixel.2;
    }
}
