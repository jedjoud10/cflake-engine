use rendering::{
    CameraUniform, DefaultMaterialResources, Indirect,
    Material, Renderer, SceneUniform, ShadowMap,
    ShadowMapping, ShadowUniform, ActiveScenePipeline,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics,
    ModuleVisibility, PrimitiveConfig, PushConstantLayout,
    PushConstants, Shader, VertexModule, WindingOrder,
};
use utils::Time;


// Terrain shader that contains physically based lighting, but suited for terrain rendering
pub struct TerrainMaterial {
    // PBR Parameters
    pub bumpiness: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub ambient_occlusion: f32,
}

impl Material for TerrainMaterial {
    type Resources<'w> = (world::Read<'w, ShadowMapping>, world::Read<'w, Time>);

    // Load the terrain material shaders and compile them
    fn shader(graphics: &Graphics, assets: &Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>(
                "engine/shaders/scene/terrain/terrain.vert",
            )
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>(
                "engine/shaders/scene/terrain/terrain.frag",
            )
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");

        // Shadow parameters
        compiler.use_uniform_buffer::<ShadowUniform>("shadow_parameters");
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("shadow_lightspace_matrices");
    
        // Define the types for the user textures
        compiler.use_sampled_texture::<ShadowMap>("shadow_map");

        // Define the push ranges used by push constants
        compiler.use_push_constant_layout(
            PushConstantLayout::split(
                <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                <vek::Rgba<f32> as GpuPod>::size() + f32::size(),
            )
            .unwrap(),
        );

        // Compile the modules into a shader
        Shader::new(vert, frag, compiler).unwrap()
    }

    // Terrain only needs positions and normals
    fn attributes() -> rendering::MeshAttributes {
        rendering::MeshAttributes::POSITIONS
    }

    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Cw,
            cull_face: Some(graphics::Face::Front),
            wireframe: false,
        }
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        (world.get::<ShadowMapping>().unwrap(), world.get::<Time>().unwrap())
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
            .set_uniform_buffer("shadow_parameters", &resources.0.parameter_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("shadow_lightspace_matrices", &resources.0.lightspace_buffer, ..)
            .unwrap();

        // Set the scene shadow map
        group
            .set_sampled_texture("shadow_map", &resources.0.depth_tex)
            .unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        constants: &mut PushConstants<ActiveScenePipeline>,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();

        // Convert the material parameters into a vec4
        let vector = vek::Vec4::new(
            self.bumpiness,
            self.metallic,
            self.ambient_occlusion,
            self.roughness,
        );

        // Send the raw fragment bytes to the GPU
        let bytes = GpuPod::into_bytes(&vector);
        constants
            .push(bytes, 0, ModuleVisibility::Fragment)
            .unwrap();

        // Calculate "fade" effect
        let duration =  resources.1.frame_start().saturating_duration_since(renderer.instant_initialized.unwrap());
        let fade = duration.as_secs_f32().clamp(0.0, 1.0);

        // Upload the fade effect to GPU
        let bytes2 = GpuPod::into_bytes(&fade);
        constants
            .push(bytes2, bytes.len() as u32, ModuleVisibility::Fragment)
            .unwrap();
    }

    type RenderPath = Indirect;
}
