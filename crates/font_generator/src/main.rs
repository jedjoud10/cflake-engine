use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

use byteorder::{LittleEndian, WriteBytesExt};
use fonts::FontChar;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel};

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
    let original_dimension = (dimension.0 as u16, dimension.1 as u16);
    // Get the writer
    let output_file = OpenOptions::new().create(true).truncate(true).write(true).open(output_file_path).unwrap();
    let mut output_file_writer = BufWriter::new(output_file);

    // How many times to we downsample the texture?
    const DOWNSAMPLE_FACTOR: u16 = 2;
    // The threshold to detect lit pixels
    const THRESHOLD: u8 = 128;
    // The SDF max width
    const MAX_SDF_WIDTH: f32 = 10.0;

    // Write the width and height
    output_file_writer.write_u16::<LittleEndian>(original_dimension.0 / DOWNSAMPLE_FACTOR).unwrap();
    output_file_writer.write_u16::<LittleEndian>(original_dimension.1 / DOWNSAMPLE_FACTOR).unwrap();

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

    // Turn each pixel into a single bit first of all
    let bit_pixels: Vec<(u32, u32, bool)> = texture.pixels().map(|x| (x.0, x.1, x.2[0] > THRESHOLD)).collect();

    // The edited pixels, you could say
    let mut edited_pixels: Vec<Vec<(u32, u32, u8)>> = vec![vec![(0, 0, 0); original_dimension.1 as usize]; original_dimension.0 as usize];
    for font_char in font_chars.iter() {
        // Get the sub-texture
        let sub_texture = bit_pixels
            .iter()
            .filter_map(|x| {
                let valid = (x.0 as u16) > font_char.min.x && (x.1 as u16) > font_char.min.y && (x.0 as u16) < font_char.max.x && (x.1 as u16) < font_char.max.y;
                if valid {
                    Some((x.0.clone(), x.1.clone(), x.2.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<(u32, u32, bool)>>();
        for pixel in sub_texture.iter() {
            let coords = veclib::Vector2::<f32>::new(pixel.0 as f32, pixel.1 as f32);
            let pixel_color = if !pixel.2 {
                // Keep the best distance
                let mut best_distance: f32 = f32::MAX;
                // Get the distance to lit pixels
                for sdf_pixel in sub_texture.iter() {
                    if sdf_pixel.2 {
                        let sdf_coords = veclib::Vector2::<f32>::new(sdf_pixel.0 as f32, sdf_pixel.1 as f32);
                        best_distance = best_distance.min(coords.distance(sdf_coords));
                    }
                }
                if best_distance != f32::MAX {
                    // Turn the distance into a number with a range of 0, 1
                    let factor = 1.0 - (best_distance / MAX_SDF_WIDTH).clamp(0.0, 1.0);
                    (factor * 128.0) as u8
                } else {
                    0
                }
            } else {
                // This pixel is already lit
                // Keep the best distance
                let mut best_distance: f32 = f32::MAX;
                // Get the distance to unlit pixels
                for sdf_pixel in sub_texture.iter() {
                    if !sdf_pixel.2 {
                        let sdf_coords = veclib::Vector2::<f32>::new(sdf_pixel.0 as f32, sdf_pixel.1 as f32);
                        best_distance = best_distance.min(coords.distance(sdf_coords));
                    }
                }
                if best_distance != f32::MAX {
                    best_distance = best_distance.max(1.41421) - 1.41421;
                    // Turn the distance into a number with a range of 0, 1
                    let factor = (best_distance / MAX_SDF_WIDTH).clamp(0.0, 1.0) + 0.5;
                    (factor * 255.0) as u8
                } else {
                    0
                }
            };
            let mut_y = edited_pixels.get_mut(pixel.1 as usize).unwrap();
            mut_y[pixel.0 as usize] = (pixel.0, pixel.1, pixel_color);
        }
        println!("Finished creating the SDF for the character {}", font_char.id);
    }
    // The texture that will be downscaled
    let mut new_texture = DynamicImage::new_rgba8(original_dimension.0 as u32, original_dimension.1 as u32);
    // Write each new pixel to the texture
    for x_row in edited_pixels {
        for pixel in x_row {
            let pixel_color = image::Rgba([pixel.2, 0, 0, 0]);
            new_texture.put_pixel(pixel.0, pixel.1, pixel_color);
        }
    }

    // Downscale
    let new_texture = new_texture.resize_exact(
        (original_dimension.0 / DOWNSAMPLE_FACTOR) as u32,
        (original_dimension.1 / DOWNSAMPLE_FACTOR) as u32,
        image::imageops::FilterType::Triangle,
    );
    let bytes = new_texture.pixels().map(|x| x.2[0]).collect::<Vec<u8>>();

    // Debug
    println!(
        "Length: {}, original dimensions: {:?}, new dimensions: {:?}, char count: {}",
        bytes.len(),
        &dimension,
        new_texture.dimensions(),
        font_chars.len()
    );

    // Actually writing the bytes to the file
    for &byte in bytes.iter() {
        output_file_writer.write_u8(byte).unwrap();
    }

    // Write the number of characters
    output_file_writer.write_u8(font_chars.len() as u8).unwrap();

    // Write the character configs
    for font_char in font_chars.iter() {
        // Write the ID first
        output_file_writer.write_u8(font_char.id).unwrap();
        // Then you can write the min-max values
        output_file_writer.write_u16::<LittleEndian>(font_char.min.x / DOWNSAMPLE_FACTOR).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.min.y / DOWNSAMPLE_FACTOR).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.max.x / DOWNSAMPLE_FACTOR).unwrap();
        output_file_writer.write_u16::<LittleEndian>(font_char.max.y / DOWNSAMPLE_FACTOR).unwrap();
    }
}
