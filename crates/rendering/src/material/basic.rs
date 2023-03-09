use std::any::TypeId;

use crate::{
    AlbedoMap, AlbedoTexel, CameraBuffer, CameraUniform,
    DefaultMaterialResources, EnabledMeshAttributes, Material,
    NormalMap, NormalTexel, Renderer, SceneBuffer, SceneUniform,
    TimingBuffer, TimingUniform, ShadowMapping, ShadowTexel, ShadowMap, ShadowUniform,
};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    BindGroup, Compiled, Compiler, FragmentModule,
    Graphics, Normalized, PushConstants, Sampler, Shader, Texture,
    Texture2D, UniformBuffer, ValueFiller, VertexModule, RGBA,
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
        world::Read<'w, ShadowMapping>,
    );

    // Load the respective Basic shader modules and compile them
    fn shader(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>(
                "engine/shaders/scene/basic/basic.vert",
            ).unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>(
                "engine/shaders/scene/basic/basic.frag",
            ).unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets);
        compiler.use_ubo::<CameraUniform>("camera");
        compiler.use_ubo::<SceneUniform>("scene");
        compiler.use_ubo::<ShadowUniform>("shadow");
        compiler.use_fill_ubo("material");

        // Define the type layouts for the textures and samplers
        compiler.use_texture::<AlbedoMap>("gradient_map");
        compiler.use_texture::<ShadowMap>("shadow_map");
        compiler.use_texture::<AlbedoMap>("albedo_map");
        compiler.use_texture::<NormalMap>("normal_map");

        // Compile the modules into a shader
        Shader::new(
            graphics,
            vert,
            frag,
            compiler
        ).unwrap()
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
        let shadow = world.get::<ShadowMapping>().unwrap();
        (albedo_maps, normal_maps, shadow)
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
        group.set_buffer("shadow", &resources.2.buffer).unwrap();

        // Set the scene sky texture
        group
            .set_texture("gradient_map", default.sky_gradient)
            .unwrap();

        // Set the scene shadow map
        group
            .set_texture("shadow_map", &resources.2.depth_tex)
            .unwrap();
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'r, 'w>(
        &self,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        let (albedo_maps, normal_maps, _) = resources;

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
        group.set_texture("normal_map", normal_map).unwrap();

        // Fill the material UBO with the specified fields automatically
        group
            .fill_ubo("material", |fill| {
                fill.set("bumpiness", self.bumpiness).unwrap();
                fill.set("tint", self.tint).unwrap();
            })
            .unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        push_constants: &mut PushConstants,
    ) {
        push_constants
            .set("matrix", renderer.matrix.cols)
            .unwrap();
    }
}
