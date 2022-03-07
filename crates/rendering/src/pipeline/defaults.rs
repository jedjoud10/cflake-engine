use assets::assetc;
use veclib::vec2;

use crate::basics::{
    material::{Material, MaterialTextures},
    mesh::{Mesh, Vertices},
    shader::{Shader, ShaderInitSettings},
    texture::{Texture, TextureBuilder, TextureDimensions},
    uniforms::StoredUniforms,
};

use super::{Handle, Pipeline};

// Some default pipeline elements
#[derive(Default)]
pub struct DefaultElements {
    // Textures
    pub white: Handle<Texture>,
    pub black: Handle<Texture>,
    pub missing: Handle<Texture>,
    pub normal_map: Handle<Texture>,

    // Meshes
    pub mesh: Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub sphere: Handle<Mesh>,

    // Materials
    pub material: Handle<Material>,

    // Default rendering shader
    pub shader: Handle<Shader>,
}

impl DefaultElements {
    // Load the default elements
    pub(crate) fn new(pipeline: &mut Pipeline) -> Self {
        // Default textures that are created at runtime
        let white = TextureBuilder::default()
            .bytes(vec![255, 255, 255, 255])
            .dimensions(TextureDimensions::Texture2d(vec2(1, 1)))
            .build();
        let white = pipeline.textures.insert(white);

        let black = TextureBuilder::default()
            .bytes(vec![0, 0, 0, 255])
            .dimensions(TextureDimensions::Texture2d(vec2(1, 1)))
            .build();
        let black = pipeline.textures.insert(black);

        let normal_map = TextureBuilder::default()
            .bytes(vec![127, 127, 255, 255])
            .dimensions(TextureDimensions::Texture2d(vec2(1, 1)))
            .build();
        let normal_map = pipeline.textures.insert(normal_map);

        // Load the missing texture. Might seem a bit counter-intuitive but it's fine since we embed it directly into the engine
        let missing = TextureBuilder::new(assetc::load::<Texture>("defaults/textures/missing.png").unwrap()).build();
        let missing = pipeline.textures.insert(missing);

        // Default mesh
        let mesh = Mesh::default();
        let mesh = pipeline.meshes.insert(mesh);

        // Load the default cube and sphere
        let cube = pipeline.meshes.insert(assetc::load("defaults/meshes/cube.obj").unwrap());
        let sphere = pipeline.meshes.insert(assetc::load("defaults/meshes/sphere.obj").unwrap());


        // Default rendering shader
        let shader = Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/default.vrsh.glsl")
                .source("defaults/shaders/rendering/default.frsh.glsl"),
        )
        .unwrap();
        let shader = pipeline.shaders.insert(shader);

        // Default material
        let material = Material {
            shader: shader.clone(),
            textures: MaterialTextures {
                diffuse_map: missing.clone(),
                normal_map: normal_map.clone(),
                emissive_map: black.clone(),
            },
            ..Default::default()
        };
        let material = pipeline.materials.insert(material);

        Self {
            white,
            black,
            missing,
            normal_map,
            mesh,
            cube,
            sphere,
            material,
            shader,
        }
    }
}
