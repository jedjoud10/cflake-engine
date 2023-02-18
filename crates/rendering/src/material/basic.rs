use std::any::TypeId;

use crate::{Material, EnabledMeshAttributes, TimingUniform, CameraUniform, SceneUniform, CameraBuffer, TimingBuffer, SceneBuffer, DefaultMaterialResources, AlbedoMap, NormalMap, Renderer};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized,
    Texture2D, VertexModule, Compiler, Sampler, Shader, RGBA, UniformBuffer, BindGroup,
};
use utils::{Storage, Handle};

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures used by this basic material
    pub albedo_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,

    // Simple Basic Parameters
    pub roughness: f32,
    pub tint: vek::Rgb<f32>, 
}

impl Material for Basic {    
    type Resources<'w> = (
        world::Read<'w, Storage<AlbedoMap>>,
        world::Read<'w, Storage<NormalMap>>
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
    fn fetch<'w>(
        world: &'w world::World
    ) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
        (albedo_maps, normal_maps)
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'w>(
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'w>,
        group: &mut BindGroup<'w>,
    ) {
        // Set the required common buffers
        group.set_buffer("", default.camera_buffer);
        //bindings.set_buffer(default.timing_buffer);
        //bindings.set_buffer(default.scene_buffer);
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'w>(
        &self,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
        group: &BindGroup<'w>,
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
        //bindings.set_sampler(albedo_map);
        //bindings.set_sampler(normal_map);
    }

    // Set the surface bindings that will change from entity to entity
    fn set_surface_bindings<'w>(
        renderer: &Renderer,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
        group: &BindGroup<'w>,
    ) {
        
    }
}