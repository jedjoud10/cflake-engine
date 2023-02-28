use assets::Assets;
use egui::{ImageData, TextureId, TexturesDelta, ClippedPrimitive};
use graphics::{Graphics, Window, RenderPass, ColorOperations, Operation, LoadOp, StoreOp, VertexConfig, PrimitiveConfig, FragmentModule, VertexModule, Compiler, Shader, VertexInput, XY, PerVertex, XYZW, Normalized, VertexBuffer, GpuPod, BufferMode, BufferUsage, TriangleBuffer, ValueFiller};
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
        dbg!(shader.reflected());

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
        // Clear most of the buffers since we will write to them
        self.positions.clear().unwrap();
        self.texcoords.clear().unwrap();
        self.colors.clear().unwrap();
        self.triangles.clear().unwrap();

        // Cached vectors to minimize GPU commands
        let mut positions = Vec::<vek::Vec2<f32>>::new();
        let mut texcoords = Vec::<vek::Vec2<f32>>::new();
        let mut colors = Vec::<vek::Vec4<u8>>::new();
        let mut triangles = Vec::<u32>::new();

        // Convert the clipped primitives to their raw vertex representations
        // TODO: Optimize these shenanigans
        for primitive in primitives.iter() {
            match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    triangles.extend_from_slice(&mesh.indices);
                    for vertex in mesh.vertices.iter() {
                        let pos = vek::Vec2::new(vertex.pos.x, vertex.pos.y);
                        let uvs = vek::Vec2::new(vertex.uv.x, vertex.uv.y);
                        let color = vek::Vec4::new(vertex.color.r(), vertex.color.g(), vertex.color.b(), vertex.color.a());
                        positions.push(pos);
                        texcoords.push(uvs);
                        colors.push(color);
                    }
                },
                egui::epaint::Primitive::Callback(_) => {},
            }
        }

        // Write to the buffers
        dbg!(&positions);
        self.positions.extend_from_slice(&positions).unwrap();
        self.texcoords.extend_from_slice(&texcoords).unwrap();
        self.colors.extend_from_slice(&colors).unwrap();
        self.triangles.extend_from_slice(bytemuck::cast_slice(&triangles)).unwrap();

        let extent = window.size();
        let dst = window.as_render_target().unwrap();
        let mut encoder = graphics.acquire();

        // Begin the render pass
        let mut render_pass = 
            self.render_pass.begin(&mut encoder, dst, ()).unwrap();

        // Bind the graphics pipeline
        let mut active = render_pass.bind_pipeline(&self.pipeline);

        // Set the required shader uniforms
        active.set_bind_group(0, move |group| {
            group.fill_ubo("window", |fill| {
                fill.set("width", extent.w).unwrap();
                fill.set("height", extent.h).unwrap();
            }).unwrap();
        });

        // Keep track of the vertex and triangle offset
        let mut vertex_offset = 0;
        let mut triangle_offset = 0;

        // Bind all the buffers, and execute all the draw commands
        for primitive in primitives.iter() {
            match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    let verts = mesh.vertices.len();
                    let triangles = mesh.indices.len() / 3;

                    active.set_vertex_buffer::<XY<f32>>(0, &self.positions, vertex_offset..(vertex_offset + verts));
                    active.set_vertex_buffer::<XY<f32>>(1, &self.texcoords, vertex_offset..(vertex_offset + verts));
                    active.set_vertex_buffer::<XYZW<Normalized<u8>>>(2, &self.colors, vertex_offset..(vertex_offset + verts));
                    active.set_index_buffer(&self.triangles, triangle_offset..(triangle_offset + triangles));
                    active.draw_indexed(0..(triangles as u32 * 3), 0..1);

                    vertex_offset += verts;
                    triangle_offset += triangles;
                },
                egui::epaint::Primitive::Callback(_) => {},
            }
        }


        // Submit the encoder at the end
        drop(render_pass);
        graphics.submit([encoder]);
    }
}
