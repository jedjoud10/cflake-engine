use crate::{
    CameraUniform, DefaultMaterialResources, Direct, Material, Renderer, Pass,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics, ModuleVisibility, PushConstantLayout,
    PushConstants, Shader, VertexModule, ActiveRenderPipeline,
};


// OVerly simplistic wireframe material
pub struct WireframeMaterial;

impl Material for WireframeMaterial {
    type Resources<'w> = ();
    type RenderPath = Direct;
    type Settings<'s> = ();
    type Query<'a> = &'a ();
    
    // Load the respective Wireframe shader modules and compile them
    fn shader<P: Pass>(_settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Option<Shader> {
        if P::is_shadow_pass() {
            return None
        }

        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>("engine/shaders/scene/wireframe/wireframe.vert")
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>("engine/shaders/scene/wireframe/wireframe.frag")
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");

        // Define the push ranges used by push constants
        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                ModuleVisibility::Vertex,
            )
            .unwrap(),
        );

        // Compile the modules into a shader
        Some(Shader::new(vert, frag, &compiler).unwrap())
    }

    // Only the positions are required for wireframe meshes
    fn attributes<P: Pass>() -> crate::MeshAttributes {
        crate::MeshAttributes::POSITIONS
    }

    // Activate the wireframe mode
    fn primitive_config() -> graphics::PrimitiveConfig {
        graphics::PrimitiveConfig::Triangles {
            winding_order: graphics::WindingOrder::Cw,
            cull_face: None,
            wireframe: true,
        }
    }

    // Fetch the texture storages
    fn fetch<P: Pass>(_world: &world::World) -> Self::Resources<'_> { }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, P: Pass>(
        _resources: &'r mut Self::Resources<'_>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w, P: Pass>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        constants: &mut PushConstants<ActiveRenderPipeline<P::C, P::DS>>,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();
    }
}
