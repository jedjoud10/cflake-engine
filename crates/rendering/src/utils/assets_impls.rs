use crate::basics::model::Model;
use crate::basics::texture::{Texture, TextureFormat, TextureType};

use crate::utils::*;

use assets::Asset;
use image::{EncodableLayout, GenericImageView};

// All the Asset trait implementations are here
// One for the textures
impl Asset for Texture {
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Read bytes
        pub fn read_bytes(metadata: &assets::AssetMetadata) -> (Vec<u8>, u16, u16) {
            // Load this texture from the bytes
            let png_bytes = metadata.bytes.as_bytes();
            let image = image::load_from_memory(png_bytes).unwrap();
            let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
            // Flip
            let image = image.flipv();
            (image.to_bytes(), image.width() as u16, image.height() as u16)
        }
        // Load this texture from the bytes
        let (bytes, width, height) = read_bytes(data);

        // Return a texture with the default parameters
        let texture = self
            .set_bytes(bytes)
            .set_dimensions(TextureType::Texture2D(width, height))
            .set_format(TextureFormat::RGBA8R)
            .set_data_type(DataType::U8);
        Some(texture)
    }
}

// One for the models
impl Asset for Model {
    // Load a model from an asset
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        let string = data.read_string();
        let lines = string.lines();
        let mut model = Model::default();
        for line in lines {
            let start = line.split_once(' ').unwrap().0;
            let other = line.split_once(' ').unwrap().1;
            match start {
                // Vertices
                "v" => {
                    let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                    model.vertices.push(veclib::Vector3::new(coords[0], coords[1], coords[2]));
                }
                // Normals
                "n" => {
                    let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                    model.normals.push(veclib::Vector3::new(coords[0], coords[1], coords[2]));
                }
                // UVs
                "u" => {
                    let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                    model.uvs.push(veclib::Vector2::new(coords[0], coords[1]));
                }
                // Tangents
                "t" => {
                    let coords: Vec<f32> = other.split('/').map(|coord| coord.parse::<f32>().unwrap()).collect();
                    model.tangents.push(veclib::Vector4::new(coords[0], coords[1], coords[2], coords[3]));
                }
                // Triangle indices
                "i" => {
                    // Split the triangle into 3 indices
                    let mut indices = other.split('/').map(|x| x.to_string().parse::<u32>().unwrap()).collect();
                    model.triangles.append(&mut indices);
                }
                _ => {}
            }
        }
        // ISTFG If this fixes the issue I will be so angry
        model.colors = vec![veclib::Vector3::ONE; model.vertices.len()];
        // Return
        Some(model)
    }
}
