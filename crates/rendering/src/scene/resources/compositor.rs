use assets::Assets;
use graphics::{
    Compiler, FragmentModule, Graphics, LoadOp, Operation,
    PrimitiveConfig, RenderPass, RenderPipeline, Shader, StoreOp,
    SwapchainFormat, Texture2D, VertexConfig, VertexModule,
};

use crate::{SceneColor, WindowUniform, SceneDepth, CameraUniform};

// This is what will write to the swapchain
pub type FinalRenderPass = RenderPass<SwapchainFormat, ()>;
pub type FinalRenderPipeline = RenderPipeline<SwapchainFormat, ()>;

// Overlays post-processing effects and multiple layers
// This will also render out the final composed image to the window
pub struct Compositor {
    // Display render pass, shader, and pipeline
    pub(crate) render_pass: FinalRenderPass,
    pub(crate) pipeline: FinalRenderPipeline,
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
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_sampled_texture::<Texture2D<SceneColor>>(
            "color_map",
        );
        compiler.use_sampled_texture::<Texture2D<SceneDepth>>(
            "depth_map",
        );
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<WindowUniform>("window");

        // Combine the modules to the shader
        let shader = Shader::new(vertex, fragment, compiler).unwrap();

        // Create the display render pass
        let render_pass = FinalRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0)),
                store: StoreOp::Store,
            },
            (),
        );

        // Create the display graphics pipeline
        let pipeline = FinalRenderPipeline::new(
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
            pipeline,
        }
    }
}
