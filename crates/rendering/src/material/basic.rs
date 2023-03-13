use crate::{
    AlbedoMap, CameraUniform, DefaultMaterialResources, Material,
    NormalMap, Renderer, SceneUniform, ShadowMap, ShadowMapping,
    ShadowUniform,
};

use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics,
    ModuleVisibility, PushConstants, Shader, VertexModule,
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

// Uniform that contains the data for the "Basic" material
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C, align(16))]
struct BasicUniform {
    pub bumpiness: f32,
    pub _padding: [f32; 3],
    pub tint: vek::Rgba<f32>,
}

impl Material for Basic {
    type Resources<'w> = (
        world::Read<'w, Storage<AlbedoMap>>,
        world::Read<'w, Storage<NormalMap>>,
        world::Read<'w, ShadowMapping>,
    );

    // Load the respective Basic shader modules and compile them
    fn shader(graphics: &Graphics, assets: &mut Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>(
                "engine/shaders/scene/basic/basic.vert",
            )
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>(
                "engine/shaders/scene/basic/basic.frag",
            )
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");
        compiler.use_uniform_buffer::<ShadowUniform>("shadow");

        // Set the dynamic ubo properties
        compiler.use_uniform_buffer::<BasicUniform>("material");

        // Define the types for the user textures
        compiler.use_texture::<AlbedoMap>("gradient_map");
        compiler.use_texture::<ShadowMap>("shadow_map");
        compiler.use_texture::<AlbedoMap>("albedo_map");
        compiler.use_texture::<NormalMap>("normal_map");

        // Define the push ranges used by push constants
        let size = <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size();
        compiler.use_push_constant_range(
            0..size,
            ModuleVisibility::Vertex,
        );

        // Compile the modules into a shader
        Shader::new(graphics, vert, frag, compiler).unwrap()
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
        group
            .set_uniform_buffer("camera", default.camera_buffer)
            .unwrap();
        group
            .set_uniform_buffer("scene", default.scene_buffer)
            .unwrap();
        group
            .set_uniform_buffer("shadow", &resources.2.buffer)
            .unwrap();

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
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        constants: &mut PushConstants,
    ) {
        // Send the raw bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex);
    }
}
