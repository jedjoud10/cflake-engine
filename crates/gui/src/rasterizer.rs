use assets::Assets;
use egui::{ImageData, TextureId, TexturesDelta, ClippedPrimitive};
use graphics::{Graphics, Window, RenderPass, ColorOperations, Operation, LoadOp, StoreOp, VertexConfig, PrimitiveConfig, FragmentModule, VertexModule, Compiler, Shader, VertexInput, XY, PerVertex, XYZW, Normalized, VertexBuffer, GpuPod, BufferMode, BufferUsage, TriangleBuffer};
use rendering::{FinalRenderPass, FinalGraphicsPipeline};

// A global rasterizer that will draw the Egui elements onto the screen
pub(crate) struct Rasterizer {
    // Render pass and shit needed for displaying
    render_pass: FinalRenderPass,
    shader: Shader,
    pipeline: FinalGraphicsPipeline,

    // Vertex buffer that contains ALL of the clipped meshes
    positions: VertexBuffer<XY<f32>>,
    texcoords: VertexBuffer<XY<f32>>,
    colors: VertexBuffer<XYZW<Normalized<u8>>>,
    
    // Triangle buffers oui oui oui
    triangles: TriangleBuffer<u32>,
}

fn create_vertex_buffer<V: graphics::Vertex>(
    graphics: &Graphics,
) -> VertexBuffer<V> {
    VertexBuffer::<V>::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::WRITE | BufferUsage::COPY_SRC,
    )
    .unwrap()
}

fn create_index_buffer(
    graphics: &Graphics,
) -> TriangleBuffer<u32> {
    TriangleBuffer::<u32>::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::WRITE | BufferUsage::COPY_SRC,
    )
    .unwrap()
}

impl Rasterizer {
    // Create a new rasterizer using an asset loader and a WGPU context
    pub(super) fn new(graphics: &Graphics, assets: &mut Assets) -> Self {
        // Load the vertex module for the display shader
        let vertex = assets.load::<VertexModule>(
            "engine/shaders/post/gui.vert"
        ).unwrap();
        let vertex = Compiler::new(vertex).compile(assets, graphics).unwrap();
        
        // Load the fragment module for the display shader
        let fragment = assets.load::<FragmentModule>(
            "engine/shaders/post/gui.frag"
        ).unwrap();
        let fragment = Compiler::new(fragment).compile(assets, graphics).unwrap();

        // Combine the modules to the shader
        let shader = Shader::new(graphics, &vertex, &fragment);

        // Create the render pass that will write to the swapchain
        let render_pass = FinalRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
            ()
        ).unwrap();

        // Create the appropriate vertex config for Egui
        let vertex_config = VertexConfig {
            inputs: [
                PerVertex::<XY<f32>>::info(),
                PerVertex::<XY<f32>>::info(),
                PerVertex::<XYZW<Normalized<u8>>>::info(),
            ].to_vec(),
        };

        // Create the display graphics pipeline
        let pipeline = FinalGraphicsPipeline::new(
            graphics,
            None,
            None,
            vertex_config,
            PrimitiveConfig::Triangles {
                winding_order: graphics::WindingOrder::Ccw,
                cull_face: None,
                wireframe: false,
            },
            &shader
        ).unwrap();

        Self {
            render_pass,
            shader,
            pipeline,
            positions: create_vertex_buffer::<XY<f32>>(graphics),
            texcoords: create_vertex_buffer::<XY<f32>>(graphics),
            colors: create_vertex_buffer::<XYZW<Normalized<u8>>>(graphics),
            triangles: create_index_buffer(graphics),
        }
    }

    // Draw the whole user interface onto the screen
    pub(crate) fn draw(
        &mut self,
        graphics: &Graphics,
        window: &mut Window,
        _loader: &mut Assets,
        primitives: Vec<ClippedPrimitive>,
        deltas: TexturesDelta,
    ) {
        // Convert the clipped primitives to their raw vertex representations

        let dst = window.as_render_target().unwrap();
        let mut encoder = graphics.acquire();

        // Begin the render pass
        let mut render_pass = 
            self.render_pass.begin(&mut encoder, dst, ()).unwrap();

        // Bind the graphics pipeline
        let mut active = render_pass.bind_pipeline(&self.pipeline);

        // Set the required shader uniforms

        // Submit the encoder at the end
        drop(render_pass);
        graphics.submit([encoder]);
    }
}
