use std::any::TypeId;

use crate::{Material, EnabledMeshAttributes};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized,
    Texture2D, VertexModule, Compiler, Sampler, Shader, BindingConfig, RGBA,
};
use utils::{Storage, Handle};

// Basic type aliases
type AlbedoTexel = RGBA<Normalized<u8>>;
type NormalTexel = RGBA<Normalized<i8>>;
type AlbedoMap = Texture2D<AlbedoTexel>;
type NormalMap = Texture2D<NormalTexel>;

/*
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

impl Material for Basic {
    type Resources<'w> = world::Read<'w, Storage<Box<u32>>>;

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
        //EnabledMeshAttributes::POSITIONS | EnabledMeshAttributes::NORMALS | EnabledMeshAttributes::TEX_COORDS
        //EnabledMeshAttributes::empty()
        EnabledMeshAttributes::POSITIONS 
    }

    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let storage = world.get::<Storage<Box<u32>>>().unwrap();
        storage
    }
}

*/