use assets::Assets;
use bytemuck::{Zeroable, Pod};
use graphics::{
    Compiler, FragmentModule, Graphics, LoadOp, Operation, PrimitiveConfig, RenderPass,
    RenderPipeline, Shader, StoreOp, SwapchainFormat, Texture2D, VertexConfig, VertexModule, UniformBuffer, BufferMode, BufferUsage,
};

use crate::{CameraUniform, SceneColor, SceneDepth, WindowUniform};

// This is what will write to the swapchain
pub type FinalRenderPass = RenderPass<SwapchainFormat, ()>;
pub type FinalRenderPipeline = RenderPipeline<SwapchainFormat, ()>;

// Container for post-processing parameters
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PostProcessUniform {
    // Lighting parameters
    pub exposure: f32,
    pub gamma: f32,

    // Vignette parameters
    pub vignette_strength: f32,
    pub vignette_size: f32,
}

impl Default for PostProcessUniform {
    fn default() -> Self {
        Self {
            exposure: 2.0,
            gamma: 2.2,
            vignette_strength: 0.4,
            vignette_size: 0.1,
        }
    }
}


// Overlays post-processing effects and multiple layers
// This will also render out the final composed image to the window
pub struct Compositor {
    // Display render pass, shader, and pipeline
    pub(crate) render_pass: FinalRenderPass,
    pub(crate) pipeline: FinalRenderPipeline,

    // Post processing settings and buffer
    pub post_process: PostProcessUniform,
    pub(crate) post_process_buffer: UniformBuffer<PostProcessUniform>,
}

impl Compositor {
    // Create a new compositor that will mix and match multiple screen textures
    pub(crate) fn new(graphics: &Graphics, assets: &mut Assets) -> Self {
        // Load the vertex module for the display shader
        let vertex = assets
            .load::<VertexModule>("engine/shaders/post/display.vert")
            .unwrap();

        // Load the fragment module for the display shader
        let fragment = assets
            .load::<FragmentModule>("engine/shaders/post/display.frag")
            .unwrap();

        // Create the bind layout for the compositor shader
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_sampled_texture::<Texture2D<SceneColor>>("color_map");
        compiler.use_sampled_texture::<Texture2D<SceneDepth>>("depth_map");
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<WindowUniform>("window");
        compiler.use_uniform_buffer::<PostProcessUniform>("post_processing");

        // Combine the modules to the shader
        let shader = Shader::new(vertex, fragment, &compiler).unwrap();

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

        // Create a uniform buffer that will contain post processing parameters
        let post_process_buffer =  UniformBuffer::from_slice(
            &graphics,
            &[PostProcessUniform::default()],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        ).unwrap();

        Self {
            render_pass,
            pipeline,
            post_process: PostProcessUniform::default(),
            post_process_buffer,
        }
    }
}
