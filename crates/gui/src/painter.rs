use assets::Assets;
use egui::ClippedMesh;
use egui::epaint::Vertex;
use egui::{ImageData, TextureId, TexturesDelta};

use rendering::buffer::{ArrayBuffer, BufferMode, ElementBuffer};
use rendering::canvas::{BlendMode, Factor, PrimitiveMode, RasterSettings};
use rendering::context::{Context, Window};
use rendering::gl;
use rendering::object::ToGlName;
use rendering::prelude::MipMaps;
use rendering::shader::{FragmentStage, Processor, Shader, ShaderCompiler, VertexStage};
use rendering::texture::{Filter, Ranged, Sampling, Texture, Texture2D, TextureMode, Wrap, RGBA};

use std::mem::size_of;
use std::ptr::null;

// Texel type that will be used to describe the inner raw texel that the texture will use
type Texel = RGBA<Ranged<u8>>;

// Convert some image data into the RGBA texels
fn image_data_to_texels(image: &ImageData) -> Vec<vek::Vec4<u8>> {
    match image {
        // I don't like this but I have to cope
        ImageData::Color(color) => color
            .pixels
            .iter()
            .map(|pixel| vek::Vec4::new(pixel.r(), pixel.g(), pixel.b(), pixel.a()))
            .collect::<Vec<vek::Vec4<u8>>>(),

        // Iterate through each alpha pixel and create a full color from it
        ImageData::Alpha(alpha) => {
            let mut texels = Vec::<vek::Vec4<u8>>::with_capacity(alpha.pixels.len() * 4);
            for alpha in alpha.pixels.iter() {
                texels.push(vek::Vec4::broadcast(*alpha));
            }
            texels
        }
    }
}

// A global rasterizer that will draw the eGUI elements onto the screen canvas
pub struct Painter {
    // A simple 2D shader that will draw the shapes
    shader: Shader,

    // Main font texture
    texture: Option<Texture2D<Texel>>,

    // The VAO for the whole rasterizer mesh
    vao: u32,

    // Dynamic buffers that we will update each frame
    indices: ElementBuffer<u32>,
    vertices: ArrayBuffer<Vertex>,
}

impl Painter {
    // Create a new rasterizer using an asset loader an OpenGL context
    pub(super) fn new(loader: &mut Assets, ctx: &mut Context) -> Self {
        // Load the shader stages first, then compile a shader
        let vert = loader
            .load::<VertexStage>("engine/shaders/gui.vrsh.glsl")
            .unwrap();
        let frag = loader
            .load::<FragmentStage>("engine/shaders/gui.frsh.glsl")
            .unwrap();

        // Link the stages and compile the shader
        let shader = ShaderCompiler::link((vert, frag), Processor::from(loader), ctx);

        // Create the main mesh VAO
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // Resizable buffers for vertices and indices
        let vertices = ArrayBuffer::<Vertex>::from_slice(ctx, &[], BufferMode::Resizable).unwrap();
        let indices = ElementBuffer::<u32>::from_slice(ctx, &[], BufferMode::Resizable).unwrap();

        // Set the vertex attribute parameters for the position, uv, and color attributes
        unsafe {
            const STRIDE: i32 = size_of::<Vertex>() as i32;
            gl::BindBuffer(gl::ARRAY_BUFFER, vertices.name());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, STRIDE, null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                STRIDE,
                (size_of::<f32>() * 2) as isize as _,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::FALSE,
                STRIDE,
                (size_of::<f32>() * 4) as isize as _,
            );
            gl::VertexArrayElementBuffer(vao, indices.name());
            gl::BindVertexArray(0);
        }

        Self {
            shader,
            texture: None,
            vao,
            indices,
            vertices,
        }
    }

    // Draw the whole user interface onto the screen
    pub fn draw(
        &mut self,
        window: &mut Window,
        ctx: &mut Context,
        meshes: Vec<ClippedMesh>,
        _loader: &mut Assets,
        deltas: TexturesDelta,
    ) {
        // Update font texture
        if let Some((_tid, delta)) = deltas
            .set
            .iter()
            .find(|(&tid, _)| tid == TextureId::Managed(0))
        {
            // Insert the texture if we don't have it already
            self.texture.get_or_insert_with(|| {
                let dimensions = vek::Extent2::from_slice(&delta.image.size()).as_::<u16>();
                let texels = image_data_to_texels(&delta.image);

                // Create the main font texture since it is missing
                Texture2D::new(
                    ctx,
                    TextureMode::Resizable,
                    dimensions,
                    Sampling {
                        filter: Filter::Nearest,
                        wrap: Wrap::ClampToEdge,
                    },
                    MipMaps::Disabled,
                    &texels,
                )
                .unwrap()
            });
        }

        // Setup OpenGL settings like blending settings and all
        let settings = RasterSettings {
            depth_test: None,
            scissor_test: None,
            primitive: PrimitiveMode::Triangles { cull: None },
            srgb: true,
            blend: Some(BlendMode {
                src: Factor::One,
                dest: Factor::OneMinusSrcAlpha,
            }),
        };

        // Create a new canvas rasterizer and fetch it's uniforms
        let (mut rasterizer, mut uniforms) =
            window
                .canvas_mut()
                .rasterizer(ctx, &mut self.shader, settings);

        // Set the global static uniforms at the start
        let texture = self.texture.as_ref().unwrap();
        uniforms.set_sampler("u_sampler", texture);
        uniforms.set_vec2::<vek::Vec2<i32>>(
            "resolution",
            rasterizer.canvas().size().as_::<i32>().into(),
        );

        for mesh in meshes {
            self.vertices.clear();
            self.indices.clear();
            
            self.vertices.extend_from_slice(mesh.1.vertices.as_slice());
            self.indices.extend_from_slice(mesh.1.indices.as_slice());

            unsafe {
                rasterizer
                    .draw_vao_elements(
                        self.vao,
                        self.indices.len(),
                        gl::UNSIGNED_INT,
                        uniforms.validate().unwrap(),
                    );
            }
        }
    }
}
