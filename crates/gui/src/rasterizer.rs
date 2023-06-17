use assets::Assets;
use egui::{ClippedPrimitive, ImageData, TextureId, TexturesDelta};
use graphics::{
    ActivePipeline, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferMode,
    BufferUsage, Compiler, FragmentModule, Graphics, LoadOp, Normalized, Operation,
    PerVertex, PrimitiveConfig, SamplerFilter, SamplerMipMaps, SamplerSettings, SamplerWrap,
    Shader, StoreOp, Texture, Texture2D, TextureMipMaps, TextureMode, TextureUsage, TriangleBuffer,
    VertexBuffer, VertexConfig, VertexInput, VertexModule, Window, RGBA, XY, XYZW, TextureViewSettings,
};
use rendering::{FinalRenderPass, FinalRenderPipeline, WindowBuffer, WindowUniform};

// Font texel type and font map
type FontTexel = RGBA<Normalized<u8>>;
type FontMap = Texture2D<FontTexel>;
type FontMapRegion = <FontMap as Texture>::Region;
// A global rasterizer that will draw the Egui elements onto the screen
pub(crate) struct Rasterizer {
    // Render pass and shit needed for displaying
    render_pass: FinalRenderPass,
    pipeline: FinalRenderPipeline,

    // Vertex buffer that contains ALL of the clipped meshes
    positions: VertexBuffer<XY<f32>>,
    texcoords: VertexBuffer<XY<f32>>,
    colors: VertexBuffer<XYZW<Normalized<u8>>>,

    // Triangle buffers oui oui oui
    triangles: TriangleBuffer<u32>,

    // Egui font texture
    texture: Option<FontMap>,
}

fn create_vertex_buffer<V: graphics::Vertex>(graphics: &Graphics) -> VertexBuffer<V> {
    VertexBuffer::<V>::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::WRITE | BufferUsage::COPY_SRC,
    )
    .unwrap()
}

fn create_index_buffer(graphics: &Graphics) -> TriangleBuffer<u32> {
    TriangleBuffer::<u32>::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::WRITE | BufferUsage::COPY_SRC,
    )
    .unwrap()
}

fn create_rf32_texture(
    graphics: &Graphics,
    extent: vek::Extent2<u32>,
    texels: &[f32],
) -> Texture2D<FontTexel> {
    let texels = texels
        .iter()
        .map(|x| vek::Vec4::broadcast(x * u8::MAX as f32).as_::<u8>())
        .collect::<Vec<_>>();

    Texture2D::from_texels(
        graphics,
        Some(&texels),
        extent,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        &[TextureViewSettings::whole::<FontMapRegion>()],
        Some(SamplerSettings {
            mipmaps: SamplerMipMaps::Auto,
            mag_filter: SamplerFilter::Linear,
            min_filter: SamplerFilter::Linear,
            wrap_u: SamplerWrap::ClampToEdge,
            wrap_v: SamplerWrap::ClampToEdge,
            ..Default::default()
        }),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

impl Rasterizer {
    // Create a new rasterizer using an asset loader and a WGPU context
    pub(super) fn new(graphics: &Graphics, assets: &mut Assets) -> Self {
        let vertex = assets
            .load::<VertexModule>("engine/shaders/gui/gui.vert")
            .unwrap();

        let fragment = assets
            .load::<FragmentModule>("engine/shaders/gui/gui.frag")
            .unwrap();

        // Create the bind layout for the GUI shader
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_sampled_texture::<FontMap>("font", false);
        compiler.use_uniform_buffer::<WindowUniform>("window");
        let shader = Shader::new(vertex, fragment, &compiler).unwrap();

        // Create the render pass that will write to the swapchain
        let render_pass = FinalRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
            (),
        );

        // Create the appropriate vertex config for Egui
        let vertex_config = VertexConfig {
            inputs: [
                PerVertex::<XY<f32>>::info(0),
                PerVertex::<XY<f32>>::info(1),
                PerVertex::<XYZW<Normalized<u8>>>::info(2),
            ]
            .to_vec(),
        };

        // Create the display graphics pipeline
        let pipeline = FinalRenderPipeline::new(
            graphics,
            None,
            None,
            Some([Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    src_factor: BlendFactor::OneMinusDstAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
            })]),
            vertex_config,
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
            positions: create_vertex_buffer::<XY<f32>>(graphics),
            texcoords: create_vertex_buffer::<XY<f32>>(graphics),
            colors: create_vertex_buffer::<XYZW<Normalized<u8>>>(graphics),
            triangles: create_index_buffer(graphics),
            texture: None,
        }
    }

    // Draw the whole user interface onto the screen
    pub(crate) fn draw(
        &mut self,
        graphics: &Graphics,
        window_buffer: &WindowBuffer,
        window: &mut Window,
        _loader: &mut Assets,
        primitives: Vec<ClippedPrimitive>,
        deltas: TexturesDelta,
    ) {
        if let Some((_, delta)) = deltas
            .set
            .iter()
            .find(|(tid, _)| *tid == TextureId::Managed(0))
        {
            // Insert the texture if we don't have it already
            self.texture.get_or_insert_with(|| {
                let dimensions = vek::Extent2::from_slice(&delta.image.size()).as_::<u32>();

                // For now, we only support the font texture
                match &delta.image {
                    ImageData::Font(font) => {
                        create_rf32_texture(graphics, dimensions, &font.pixels)
                    }
                    _ => todo!(),
                }
            });
        }

        // Clear most of the buffers since we will write to them
        self.positions.clear().unwrap();
        self.texcoords.clear().unwrap();
        self.colors.clear().unwrap();
        self.triangles.clear().unwrap();

        // Cached vectors to minimize GPU commands
        let mut positions = Vec::<vek::Vec2<f32>>::with_capacity(self.positions.capacity());
        let mut texcoords = Vec::<vek::Vec2<f32>>::with_capacity(self.texcoords.capacity());
        let mut colors = Vec::<vek::Vec4<u8>>::with_capacity(self.colors.capacity());
        let mut triangles = Vec::<u32>::with_capacity(self.triangles.capacity());

        // Convert the clipped primitives to their raw vertex representations
        for primitive in primitives.iter() {
            match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    triangles.extend_from_slice(&mesh.indices);
                    for vertex in mesh.vertices.iter() {
                        let pos = vek::Vec2::new(vertex.pos.x, vertex.pos.y);
                        let uvs = vek::Vec2::new(vertex.uv.x, vertex.uv.y);
                        let color = vek::Vec4::from_slice(&vertex.color.to_array());
                        positions.push(pos);
                        texcoords.push(uvs);
                        colors.push(color);
                    }
                }
                egui::epaint::Primitive::Callback(_) => {}
            }
        }

        // Write to the buffers
        self.positions.extend_from_slice(&positions).unwrap();
        self.texcoords.extend_from_slice(&texcoords).unwrap();
        self.colors.extend_from_slice(&colors).unwrap();
        let triangles = bytemuck::cast_slice(&triangles);
        self.triangles.extend_from_slice(triangles).unwrap();

        // Get the destination render target we will render to
        let Ok(dst) = window.as_render_target() else {
            return;
        };

        // Begin the render pass and bing the GUI pipeline
        let mut render_pass = self.render_pass.begin(dst, ());
        let mut active = render_pass.bind_pipeline(&self.pipeline);

        // Set the required shader uniforms
        let texture = self.texture.as_ref().unwrap();
        active
            .set_bind_group(0, |group| {
                group
                    .set_uniform_buffer("window", window_buffer, ..)
                    .unwrap();
                group.set_sampled_texture_view("font", texture).unwrap();
            })
            .unwrap();

        // Keep track of the vertex and triangle offset
        let mut vertex_offset = 0;
        let mut triangle_offset = 0;

        // Bind all the buffers, and execute all the draw commands
        for primitive in primitives.iter() {
            match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    let verts = mesh.vertices.len();
                    let triangles = mesh.indices.len() / 3;

                    active
                        .set_vertex_buffer::<XY<f32>>(
                            0,
                            &self.positions,
                            vertex_offset..(vertex_offset + verts),
                        )
                        .unwrap();
                    active
                        .set_vertex_buffer::<XY<f32>>(
                            1,
                            &self.texcoords,
                            vertex_offset..(vertex_offset + verts),
                        )
                        .unwrap();
                    active
                        .set_vertex_buffer::<XYZW<Normalized<u8>>>(
                            2,
                            &self.colors,
                            vertex_offset..(vertex_offset + verts),
                        )
                        .unwrap();
                    active
                        .set_index_buffer(
                            &self.triangles,
                            triangle_offset..(triangle_offset + triangles),
                        )
                        .unwrap();
                    active
                        .draw_indexed(0..(triangles as u32 * 3), 0..1)
                        .unwrap();

                    vertex_offset += verts;
                    triangle_offset += triangles;
                }
                egui::epaint::Primitive::Callback(_) => {}
            }
        }

        // Submit the encoder at the end
        drop(render_pass);
    }
}
