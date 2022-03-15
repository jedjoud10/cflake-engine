use assets::assetc;

use crate::basics::{
    material::{Material, MaterialBuilder, PbrMaterialBuilder, PbrParams, PbrTextures},
    mesh::Mesh,
    shader::{Shader, ShaderInitSettings},
    texture::{Texture, Texture2D, TextureBuilder, TextureParams, TextureFlags},
};

use super::{Handle, Pipeline};

// Some default pipeline elements
#[derive(Default)]
pub struct DefaultElements {
    // Textures
    pub white: Handle<Texture2D>,
    pub black: Handle<Texture2D>,
    pub missing: Handle<Texture2D>,
    pub normal_map: Handle<Texture2D>,

    // Meshes
    pub mesh: Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub sphere: Handle<Mesh>,

    // Materials
    pub pbr_mat: Handle<Material>,
    pub pbr_mat_white: Handle<Material>,
    pub pbr_mat_black: Handle<Material>,

    // Default rendering shader
    pub shader: Handle<Shader>,
}

impl DefaultElements {
    // Load the default elements
    pub(crate) fn new(pipeline: &mut Pipeline) -> Self {
        // Default textures that are created at runtime
        let params = TextureParams {
            flags: TextureFlags::empty(),
            ..Default::default()
        };
        let white = TextureBuilder::default().params(params.clone()).bytes(vec![255, 255, 255, 255]).dimensions(vek::Extent2::one()).build();
        let white = pipeline.insert(white);

        let black = TextureBuilder::default().params(params.clone()).bytes(vec![0, 0, 0, 255]).dimensions(vek::Extent2::one()).build();
        let black = pipeline.insert(black);

        let normal_map = TextureBuilder::default().params(params).bytes(vec![127, 127, 255, 255]).dimensions(vek::Extent2::one()).build();
        let normal_map = pipeline.insert(normal_map);

        // Load the missing texture. Might seem a bit counter-intuitive but it's fine since we embed it directly into the engine
        let missing = TextureBuilder::new(assetc::load::<Texture2D>("defaults/textures/missing.png").unwrap()).build();
        let missing = pipeline.insert(missing);

        // Default mesh
        let mesh = Mesh::default();
        let mesh = pipeline.insert(mesh);

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

        // Default pbr material (uses missing texture)
        let pbr_mat = PbrMaterialBuilder {
            textures: PbrTextures {
                diffuse: missing.clone(),
                normal: normal_map.clone(),
                emissive: black.clone(),
            },
            params: PbrParams::default(),
        }
        .build_with_shader(pipeline, shader.clone());
        let pbr_mat = pipeline.insert(pbr_mat);

        // Default pbr material (uses white texture)
        let pbr_mat_white = PbrMaterialBuilder {
            textures: PbrTextures {
                diffuse: white.clone(),
                normal: normal_map.clone(),
                emissive: black.clone(),
            },
            params: PbrParams::default(),
        }
        .build_with_shader(pipeline, shader.clone());
        let pbr_mat_white = pipeline.insert(pbr_mat_white);

        // Default pbr material (uses black texture)
        let pbr_mat_black = PbrMaterialBuilder {
            textures: PbrTextures {
                diffuse: black.clone(),
                normal: normal_map.clone(),
                emissive: black.clone(),
            },
            params: PbrParams::default(),
        }
        .build_with_shader(pipeline, shader.clone());
        let pbr_mat_black = pipeline.insert(pbr_mat_black);

        Self {
            white,
            black,
            missing,
            normal_map,
            mesh,
            cube,
            sphere,
            pbr_mat,
            pbr_mat_white,
            pbr_mat_black,
            shader,
        }
    }
}
