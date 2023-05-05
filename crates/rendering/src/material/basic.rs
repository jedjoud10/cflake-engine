use crate::{
    ActiveScenePipeline, AlbedoMap, CameraUniform, DefaultMaterialResources, Direct, Material,
    NormalMap, Renderer, SceneUniform, ShadowMap, ShadowMapping, ShadowUniform,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics, ModuleVisibility, PushConstantLayout,
    PushConstants, Shader, StorageAccess, VertexModule,
};
use utils::{Handle, Storage};

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct BasicMaterial {
    // Textures used by this basic material
    pub albedo_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,

    // Simple Basic Parameters
    pub bumpiness: f32,
    pub tint: vek::Rgb<f32>,
}

impl Material for BasicMaterial {
    type Resources<'w> = (
        world::Read<'w, Storage<AlbedoMap>>,
        world::Read<'w, Storage<NormalMap>>,
        world::Read<'w, ShadowMapping>,
    );

    type RenderPath = Direct;
    type Settings<'s> = ();
    type Query<'a> = &'a ();

    // Load the respective Basic shader modules and compile them
    fn shader(_settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>("engine/shaders/scene/basic/basic.vert")
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>("engine/shaders/scene/basic/basic.frag")
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");

        // Shadow parameters
        compiler.use_uniform_buffer::<ShadowUniform>("shadow_parameters");
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("shadow_lightspace_matrices");
        compiler.use_uniform_buffer::<f32>("cascade_plane_distances");

        // Define the types for the user textures
        compiler.use_sampled_texture::<ShadowMap>("shadow_map");
        compiler.use_sampled_texture::<AlbedoMap>("albedo_map");
        compiler.use_sampled_texture::<NormalMap>("normal_map");

        // Define the push ranges used by push constants
        compiler.use_push_constant_layout(
            PushConstantLayout::split(
                <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                <vek::Rgba<f32> as GpuPod>::size(),
            )
            .unwrap(),
        );

        // Compile the modules into a shader
        Shader::new(vert, frag, &compiler).unwrap()
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
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        // Set the required common buffers
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("scene", default.scene_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("shadow_parameters", &resources.2.parameter_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer(
                "shadow_lightspace_matrices",
                &resources.2.lightspace_buffer,
                ..,
            )
            .unwrap();
        group
            .set_uniform_buffer(
                "cascade_plane_distances",
                &resources.2.cascade_distances,
                ..,
            )
            .unwrap();

        // Set the scene shadow map
        group
            .set_sampled_texture("shadow_map", &resources.2.depth_tex)
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
        group.set_sampled_texture("albedo_map", albedo_map).unwrap();
        group.set_sampled_texture("normal_map", normal_map).unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        constants: &mut PushConstants<ActiveScenePipeline>,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();

        // Send the raw fragment bytes to the GPU
        let bytes = GpuPod::into_bytes(&self.tint);
        let offset = bytes.len() as u32;
        constants
            .push(bytes, 0, ModuleVisibility::Fragment)
            .unwrap();
        let bytes = GpuPod::into_bytes(&self.bumpiness);
        constants
            .push(bytes, offset, ModuleVisibility::Fragment)
            .unwrap();
    }
}
