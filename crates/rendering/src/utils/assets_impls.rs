use crate::basics::*;
use crate::utils::*;

use assets::{Asset, AssetObject, Object};

// All the Asset trait implementations are here
// One for the textures
impl Asset for Texture {
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let (_bytes, width, height) = Self::read_bytes(data);
        // Return a texture with the default parameters
        let texture = self
            .set_dimensions(TextureType::Texture2D(width, height))
            .set_format(TextureFormat::RGBA8R)
            .set_data_type(DataType::UByte)
            .set_name(&data.name);
        Some(texture)
    }
}

impl Object for Texture {}
impl AssetObject for Texture {}
// One for the models
impl Asset for Model {
    // Load a model from an asset
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        let string = String::from_utf8(data.bytes.clone()).unwrap();
        let lines = string.lines();
        let mut model = Model::default();
        for line in lines {
            let start = line.split_once(" ").unwrap().0;
            let other = line.split_once(" ").unwrap().1;
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

// One for the subshaders obviously
impl Asset for SubShader {
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Load a subshader from this metadata
        let text = String::from_utf8(data.bytes.clone()).ok()?;
        Some(Self {
            name: data.name.clone(),
            source: text,
            subshader_type: match &data.asset_type {
                assets::AssetType::VertSubshader => SubShaderType::Vertex,
                assets::AssetType::FragSubshader => SubShaderType::Fragment,
                assets::AssetType::ComputeSubshader => SubShaderType::Compute,
                _ => {
                    /* Nothing */
                    panic!()
                }
            },
        })
    }
}

// A subshader is also an object
impl Object for SubShader {}
