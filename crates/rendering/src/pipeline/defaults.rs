use assets::assetc;

use crate::basics::{
    material::{Material, MaterialBuilder, PbrMaterialBuilder},
    mesh::{Mesh, Vertices, Indices},
    shader::{Shader, ShaderInitSettings},
    texture::{Texture2D, TextureFilter, TextureFlags, TextureLayout, TextureParams, TextureWrapMode},
};

use super::{Handle, Pipeline};

// Some default pipeline elements
#[derive(Default)]
pub struct DefaultElements {
    // Textures
    pub white: Handle<Texture2D>,
    pub black: Handle<Texture2D>,
    pub missing_texture: Handle<Texture2D>,
    pub normal_map: Handle<Texture2D>,

    // Meshes
    pub mesh: Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub sphere: Handle<Mesh>,

    // Shader
    pub shader: Handle<Shader>,

    // Material
    pub missing_pbr_mat: Handle<Material>,
}

impl DefaultElements {
    // Load the default elements
    pub(crate) fn new(pipeline: &mut Pipeline) -> Self {
        // Default textures that are created at runtime
        let params = TextureParams {
            layout: TextureLayout::LOADED,
            filter: TextureFilter::Linear,
            wrap: TextureWrapMode::Repeat,
            flags: TextureFlags::MIPMAPS | TextureFlags::SRGB,
        };
        let white = Texture2D::new(vek::Extent2::one(), Some(vec![255, 255, 255, 255]), params);
        let white = pipeline.insert(white);

        let black = Texture2D::new(vek::Extent2::one(), Some(vec![0, 0, 0, 255]), params);
        let black = pipeline.insert(black);

        let normal_map = Texture2D::new(vek::Extent2::one(), Some(vec![127, 127, 255, 255]), params);
        let normal_map = pipeline.insert(normal_map);

        // Load the missing texture. Might seem a bit counter-intuitive but it's fine since we embed it directly into the engine
        let missing = assetc::load_with::<Texture2D>("defaults/textures/missing.png", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
        let missing_texture = pipeline.insert(missing);

        // Default empty mesh
        let mesh = pipeline.insert(Mesh::new(Vertices::default(), Indices::default()));

        // Load the default cube and sphere
        let cube = pipeline.insert(assetc::load("defaults/meshes/cube.obj").unwrap());
        let sphere = pipeline.insert(assetc::load("defaults/meshes/sphere.obj").unwrap());

        // Default rendering (PBR) shader
        let shader = Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/default.vrsh.glsl")
                .source("defaults/shaders/rendering/default.frsh.glsl"),
        )
        .unwrap();
        let shader = pipeline.insert(shader);

        // Default missing rendering (PBR) shader
        let missing_shader = Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/missing.vrsh.glsl")
                .source("defaults/shaders/rendering/missing.frsh.glsl"),
        )
        .unwrap();
        let _missing_shader = pipeline.insert(missing_shader);

        // Default pbr material (uses missing texture)
        let missing_pbr_mat = PbrMaterialBuilder::default()
            .diffuse(missing_texture.clone())
            .normal(normal_map.clone())
            .emissive(black.clone())
            .build_with_shader(pipeline, shader.clone());

        Self {
            white,
            black,
            missing_texture,
            normal_map,
            mesh,
            cube,
            sphere,
            missing_pbr_mat,
            shader,
            //missing_shader,
        }
    }
}
