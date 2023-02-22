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
    VertexModule, RGBA, Texture, ValueFiller, PushConstants,
};
use utils::{Handle, Storage};

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures used by this basic material
    pub albedo_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,

    // Simple Basic Parameters
    pub bumpiness: f32,
    pub tint: vek::Rgb<f32>,
}

impl Material for Basic {
    type Resources<'w> = (
        world::Read<'w, Storage<AlbedoMap>>,
        world::Read<'w, Storage<NormalMap>>,
    );

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

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
        (albedo_maps, normal_maps)
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        // Set the required common buffers
        group.set_buffer("camera", default.camera_buffer).unwrap();
        group.set_buffer("scene", default.scene_buffer).unwrap();
        group.set_buffer("time", default.timing_buffer).unwrap();
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'r, 'w>(
        &self,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        let (albedo_maps, normal_maps) = resources;

        // Get the albedo texture, and fallback to a white one
        let albedo_map = self
            .albedo_map
            .as_ref()
            .map_or(default.white, |h| albedo_maps.get(h));

        // Get the normal map, and fallback to the default one
        let normal_map = self
            .normal_map
            .as_ref()
            .map_or(default.normal, |h| normal_maps.get(h));
        

        // Set the material textures
        group.set_texture("albedo_map", albedo_map).unwrap();
        group.set_sampler("albedo_map_sampler", albedo_map.sampler()).unwrap();
        group.set_texture("normal_map", normal_map).unwrap();
        group.set_sampler("normal_map_sampler", normal_map.sampler()).unwrap();
        
        // Fill the material UBO with the specified fields automatically
        group.fill_buffer("material", |fill| {
            fill.set_scalar("bumpiness", self.bumpiness).unwrap();
            fill.set_vec3("tint", self.tint).unwrap();
        }).unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        push_constants: &mut PushConstants
    ) {
        push_constants.set_mat4x4("matrix", renderer.matrix.cols).unwrap();
    }
}
