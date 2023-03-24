use rendering::{
    AlbedoMap, CameraUniform, DefaultMaterialResources, Material,
    NormalMap, Renderer, SceneUniform, ShadowMap, ShadowMapping,
    ShadowUniform,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics,
    ModuleVisibility, PushConstantLayout, PushConstants, Shader,
    VertexModule,
};
use utils::{Handle, Storage};

// This is a simple terrain shader that I will use personally for debugging
// I will switch this to a proper terrain system later on (trust)
pub struct Terrain {
}

impl Material for Terrain {
    type Resources<'w> = ();

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

        // Define the push ranges used by push constants
        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                ModuleVisibility::Vertex
            )
            .unwrap(),
        );

        // Compile the modules into a shader
        Shader::new(vert, frag, compiler).unwrap()
    }

    fn attributes() -> rendering::MeshAttributes {
        rendering::MeshAttributes::POSITIONS
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        ()
    }

    fn casts_shadows() -> bool {
        true
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        // Set the required common buffers
        group
            .set_uniform_buffer("camera", default.camera_buffer)
            .unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        constants: &mut PushConstants,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();
    }
}
