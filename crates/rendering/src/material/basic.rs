use std::any::TypeId;

use crate::{Material, EnabledMeshAttributes, TimingUniform, CameraUniform, SceneUniform, CameraBuffer, TimingBuffer, SceneBuffer};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized,
    Texture2D, VertexModule, Compiler, Sampler, Shader, BindingConfig, RGBA, UniformBuffer,
};
use utils::{Storage, Handle};

// Basic type aliases
type AlbedoTexel = RGBA<Normalized<u8>>;
type NormalTexel = RGBA<Normalized<i8>>;
type AlbedoMap = Texture2D<AlbedoTexel>;
type NormalMap = Texture2D<NormalTexel>;

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures used by this basic material
    pub diffuse_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,

    // Simple Basic Parameters
    pub roughness: f32,
    pub tint: vek::Rgb<f32>, 
}

/*
impl_material_layout! {
    target: Basic,

    surface: {},

    instance: {
        #[texture(0)]
        #[fragment]
        diffuse_map: AlbedoMap,
    
        #[texture(1)]
        #[fragment]
        normal_map: NormalMap,
    
    },

    global: {
        #[buffer(0)]
        scene_buffer: SceneBuffer,
    },

    pushconstants: {
        #[uniform(pushconstant)]
        #[fragment]
        model_matrix: vek::Mat4<f32>,
    }
}
*/

impl Material for Basic {    
    type Resources<'w> = ();

    // Load the vertex shader for this material
    fn vertex(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<VertexModule> {
        let vert = assets
            .load::<VertexModule>("engine/shaders/basic.vert")
            .unwrap();
        Compiler::new(vert).compile(assets, graphics).unwrap()
    }

    // Load the fragment shader for this material
    fn fragment(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<FragmentModule> {
        let frag = assets
            .load::<FragmentModule>("engine/shaders/basic.frag")
            .unwrap();
        Compiler::new(frag).compile(assets, graphics).unwrap()
    }

    fn attributes() -> EnabledMeshAttributes {
        EnabledMeshAttributes::all()
    }
}