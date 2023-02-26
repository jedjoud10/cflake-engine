use std::any::TypeId;

use crate::{
    AlbedoMap, CameraBuffer, CameraUniform, DefaultMaterialResources,
    EnabledMeshAttributes, Material, NormalMap, Renderer,
    SceneBuffer, SceneUniform, TimingBuffer, TimingUniform,
};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    BindGroup, Compiled, Compiler, FragmentModule, Graphics,
    Normalized, Sampler, Shader, Texture2D, UniformBuffer,
    VertexModule, RGBA, Texture, ValueFiller, PushConstants, PrimitiveConfig, WindingOrder, Face,
};
use utils::{Handle, Storage};

// A very simple sky material which uses a sky color gradient
pub struct Sky {
    // sky gradient texture map
    pub gradient_map: Option<Handle<AlbedoMap>>,
}

impl Material for Sky {
    type Resources<'w> = world::Read<'w, Storage<AlbedoMap>>;

    // Load the vertex shader for this material
    fn vertex(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Compiled<VertexModule> {
        let vert = assets
            .load::<VertexModule>("engine/shaders/scene/sky/sky.vert")
            .unwrap();
        Compiler::new(vert).compile(assets, graphics).unwrap()
    }

    // Load the fragment shader for this material
    fn fragment(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Compiled<FragmentModule> {
        let frag = assets
            .load::<FragmentModule>("engine/shaders/scene/sky/sky.frag")
            .unwrap();
        Compiler::new(frag).compile(assets, graphics).unwrap()
    }

    // Get the required mesh attributes that we need to render a surface
    fn attributes() -> EnabledMeshAttributes {
        EnabledMeshAttributes::POSITIONS
    }

    // Sky-spheres are always flipped inside out
    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Ccw,
            cull_face: Some(Face::Front),
            wireframe: false,
        }
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        albedo_maps
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        group.set_buffer("camera", default.camera_buffer).unwrap();
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'r, 'w>(
        &self,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        let gradient_maps = resources;

        // Get the gradient texture, and fallback to the default one
        let albedo_map = self
            .gradient_map
            .as_ref()
            .map_or(default.sky_gradient, |h| gradient_maps.get(h));
        
        // Set the material textures
        group.set_texture("gradient_map", albedo_map).unwrap();
        group.set_sampler("gradient_map_sampler", albedo_map.sampler()).unwrap();
    }
}
