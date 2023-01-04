use crate::Material;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized, Processor,
    Texture2D, VertexModule, RGB,
};
use utils::Storage;

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures
    pub albedo_map: Texture2D<RGB<Normalized<u8>>>,
    pub normal_map: Texture2D<RGB<Normalized<i8>>>,

    // Parameters
    pub bumpiness: f32,
    pub tint: vek::Rgb<f32>,
}

impl Material for Basic {
    type Resources<'w> = world::Read<'w, Storage<Box<u32>>>;
    type SceneDescriptorSet<'ds> = &'ds Box<u32>;
    type InstanceDescriptorSet<'ds> = ();
    type SurfaceDescriptorSet<'ds> = ();

    // Load the vertex shader for this material
    fn vertex(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<VertexModule> {
        let vert = assets
            .load::<VertexModule>("engine/shaders/basic.vert")
            .unwrap();
        let processor = Processor::new(vert, &assets);
        Compiled::compile(graphics, processor.process())
    }

    // Load the fragment shader for this material
    fn fragment(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<graphics::FragmentModule> {
        let frag = assets
            .load::<FragmentModule>("engine/shaders/basic.frag")
            .unwrap();
        let processor = Processor::new(frag, &assets);
        Compiled::compile(graphics, processor.process())
    }

    fn required_mesh_attributes() -> () {
        todo!()
    }

    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let storage = world.get::<Storage<Box<u32>>>().unwrap();
        storage
    }

    fn get_static_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
    ) -> Self::SceneDescriptorSet<'ds> {
        resources.get(todo!())
    }

    fn get_surface_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
    ) -> Self::SurfaceDescriptorSet<'ds> {
        todo!()
    }

    fn get_instance_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
        instance: &Self,
    ) -> Self::InstanceDescriptorSet<'ds> {
        todo!()
    }
}
