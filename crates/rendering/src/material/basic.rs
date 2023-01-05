use crate::Material;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized,
    Texture2D, VertexModule, RGB, Compiler,
};
use utils::Storage;

// Basic type aliases
type BasicAlbedoMap = Texture2D<RGB<Normalized<u8>>>;
type BasicNormalMap = Texture2D<RGB<Normalized<i8>>>;

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures
    pub albedo_map: BasicAlbedoMap,
    pub normal_map: BasicNormalMap,

    // Parameters
    pub bumpiness: f32,
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
    ) -> Compiled<graphics::FragmentModule> {
        let frag = assets
            .load::<FragmentModule>("engine/shaders/basic.frag")
            .unwrap();
        Compiler::new(frag).compile(assets, graphics).unwrap()
    }

    fn required_mesh_attributes() -> () {
        todo!()
    }

    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let storage = world.get::<Storage<Box<u32>>>().unwrap();
        storage
    }
}