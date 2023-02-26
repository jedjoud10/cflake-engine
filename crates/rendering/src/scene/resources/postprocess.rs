use assets::Assets;
use graphics::{SwapchainFormat, RenderPass, LoadOp, StoreOp, Operation, Graphics, Shader, VertexModule, FragmentModule, Compiler, GraphicsPipeline, VertexConfig, PrimitiveConfig};

// This is what will write to the swapchain
pub type FinalRenderPass = RenderPass<SwapchainFormat, ()>;
pub type FinalGraphicsPipeline = GraphicsPipeline<SwapchainFormat, ()>;

// Container for post-processing parameters and passes
// TODO: Rename this to generic "Display" struct or smthing
pub struct PostProcess {
    // Display render pass, shader, and pipeline
    pub(crate) render_pass: FinalRenderPass,
    pub(crate) shader: Shader,
    pub(crate) pipeline: FinalGraphicsPipeline,

    // Lighting parameters
    pub exposure: f32,
    pub gamma: f32,

    // Vignette parameters
    pub vignette_strength: f32,
    pub vignette_size: f32,
} 

impl PostProcess {
    // Create a new post processing pass and default out the parameters
    pub(crate) fn new(graphics: &Graphics, assets: &mut Assets) -> Self {
        // Load the vertex module for the display shader
        let vertex = assets.load::<VertexModule>(
            "engine/shaders/post/display.vert"
        ).unwrap();
        let vertex = Compiler::new(vertex).compile(assets, graphics).unwrap();
        
        // Load the fragment module for the display shader
        let fragment = assets.load::<FragmentModule>(
            "engine/shaders/post/display.frag"
        ).unwrap();
        let fragment = Compiler::new(fragment).compile(assets, graphics).unwrap();

        // Combine the modules to the shader
        let shader = Shader::new(graphics, &vertex, &fragment);

        // Create the display render pass
        let render_pass = FinalRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0)),
                store: StoreOp::Store,
            },
            ()
        ).unwrap();

        // Create the display graphics pipeline
        let pipeline = FinalGraphicsPipeline::new(
            graphics,
            None,
            None,
            VertexConfig::default(),
            PrimitiveConfig::Triangles {
                winding_order: graphics::WindingOrder::Ccw,
                cull_face: None,
                wireframe: false,
            },
            &shader
        ).unwrap();
        
        Self {
            render_pass,
            exposure: 1.2,
            gamma: 2.2,
            vignette_strength: 1.0,
            vignette_size: 1.0,
            shader,
            pipeline,
        }
    }
}