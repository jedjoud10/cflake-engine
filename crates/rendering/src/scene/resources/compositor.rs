use assets::Assets;
use graphics::{
    Compiler, FragmentModule, Graphics, GraphicsPipeline, LoadOp,
    Operation, PrimitiveConfig, RenderPass, Shader, StoreOp,
    SwapchainFormat, Texture2D, VertexConfig, VertexModule,
};

use crate::{
    CameraUniform, SceneColor, SceneDepth, ShadowMap, WindowUniform,
};

// This is what will write to the swapchain
pub type FinalRenderPass = RenderPass<SwapchainFormat, ()>;
pub type FinalGraphicsPipeline =
    GraphicsPipeline<SwapchainFormat, ()>;

// Overlays post-processing effects and multiple layers
// This will also render out the final composed image to the window
pub struct Compositor {
    // Display render pass, shader, and pipeline
    pub(crate) render_pass: FinalRenderPass,
    pub(crate) shader: Shader,
    pub(crate) pipeline: FinalGraphicsPipeline,
}

impl Compositor {
    // Create a new compositor that will mix and match multiple screen textures
    pub(crate) fn new(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Self {
        // Load the vertex module for the display shader
        let vertex = assets
            .load::<VertexModule>("engine/shaders/post/display.vert")
            .unwrap();

        // Load the fragment module for the display shader
        let fragment = assets
            .load::<FragmentModule>(
                "engine/shaders/post/display.frag",
            )
            .unwrap();

        // Create the bind layout for the compositor shader
        let mut compiler = Compiler::new(assets);
        compiler.use_texture::<Texture2D<SceneColor>>("color_map");
        compiler.use_texture::<Texture2D<SceneDepth>>("depth_map");
        compiler.use_texture::<ShadowMap>("shadowmap");
        compiler.use_uniform_buffer::<WindowUniform>("window");
        compiler.use_uniform_buffer::<CameraUniform>("camera");

        // Combine the modules to the shader
        let shader =
            Shader::new(graphics, vertex, fragment, compiler)
                .unwrap();

        // Create the display render pass
        let render_pass = FinalRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0)),
                store: StoreOp::Store,
            },
            (),
        )
        .unwrap();

        // Create the display graphics pipeline
        let pipeline = FinalGraphicsPipeline::new(
            graphics,
            None,
            None,
            None,
            VertexConfig::default(),
            PrimitiveConfig::Triangles {
                winding_order: graphics::WindingOrder::Ccw,
                cull_face: None,
                wireframe: false,
            },
            &shader,
        )
        .unwrap();

        Self {
            render_pass,
            shader,
            pipeline,
        }
    }
}
