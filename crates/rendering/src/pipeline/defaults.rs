use crate::basics::{
    material::{Material, MaterialBuilder, PbrMaterialBuilder},
    mesh::{Indices, Mesh, Vertices},
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
    pub mask: Handle<Texture2D>,
    pub missing_texture: Handle<Texture2D>,
    pub normal_map: Handle<Texture2D>,

    // Meshes
    pub mesh: Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub plane: Handle<Mesh>,
    pub sphere: Handle<Mesh>,

    // Shaders
    pub shader: Handle<Shader>,
    pub flat: Handle<Shader>,

    // Materials
    pub missing_pbr_mat: Handle<Material>,
}

impl DefaultElements {
    // Load the default elements
    pub(crate) fn new(pipeline: &mut Pipeline) -> Self {
        // Default textures that are created at runtime
        let params = TextureParams {
            layout: TextureLayout::LOADED,
            filter: TextureFilter::Nearest,
            wrap: TextureWrapMode::Repeat,
            flags: TextureFlags::MIPMAPS,
        };
        // Create a 1x1 white texture
        let white = Texture2D::new(vek::Extent2::one(), Some(vec![255, 255, 255, 255]), params);
        let white = pipeline.insert(white);

        // Create a 1x1 black texture
        let black = Texture2D::new(vek::Extent2::one(), Some(vec![0, 0, 0, 255]), params);
        let black = pipeline.insert(black);

        // Create a 1x1 default normal map
        let normal_map = Texture2D::new(vek::Extent2::one(), Some(vec![128, 128, 255, 255]), params);
        let normal_map = pipeline.insert(normal_map);

        // Create a 1x1 default mask texture map
        let mask = Texture2D::new(vek::Extent2::one(), Some(vec![0, 255, 0, 0]), params);
        let mask = pipeline.insert(mask);

        // Load the missing texture. Might seem a bit counter-intuitive but it's fine since we embed it directly into the engine
        let missing = assets::load_with::<Texture2D>("defaults/textures/missing.png", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
        let missing_texture = pipeline.insert(missing);

        // Default empty mesh
        let mesh = pipeline.insert(Mesh::new(Vertices::default(), Indices::default()));

        // Load the default cube and sphere
        let cube = pipeline.insert(assets::load("defaults/meshes/cube.obj").unwrap());
        let sphere = pipeline.insert(assets::load("defaults/meshes/sphere.obj").unwrap());
        let plane = pipeline.insert(assets::load("defaults/meshes/plane.obj").unwrap());

        // Default rendering (PBR) shader
        let shader =  pipeline.insert(Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/default.vrsh.glsl")
                .source("defaults/shaders/rendering/default.frsh.glsl"),
        ).unwrap());

        // Default missing rendering (PBR) shader
        let missing_shader = Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/missing.vrsh.glsl")
                .source("defaults/shaders/rendering/missing.frsh.glsl"),
        )
        .unwrap();
        let _missing_shader = pipeline.insert(missing_shader);

        // Flat shaded shader (low-poly)
        let flat = pipeline.insert(Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/default.vrsh.glsl")
                .source("defaults/shaders/rendering/flat.frsh.glsl"),
        ).unwrap());

        // Default pbr material (uses missing texture)
        let missing_pbr_mat = PbrMaterialBuilder::default()
            .diffuse(missing_texture.clone())
            .normal(normal_map.clone())
            .emissive(black.clone())
            .build_with_shader(pipeline, shader.clone());

        Self {
            white,
            black,
            mask,
            missing_texture,
            normal_map,
            mesh,
            cube,
            sphere,
            plane,
            missing_pbr_mat,
            flat,
            shader,
        }
    }
}
