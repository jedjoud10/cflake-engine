use assets::loader::AssetLoader;
use egui::{epaint::Mesh, ClippedMesh, Rect};
use egui::{Color32, ImageData, TextureId, TexturesDelta};
use nohash_hasher::NoHashHasher;
use rendering::buffer::{ArrayBuffer, Buffer, BufferMode, ElementBuffer};
use rendering::canvas::rasterizer::{BlendMode, FaceCullMode, Factor, PrimitiveMode, RasterSettings};
use rendering::context::{Context, Device};
use rendering::gl;
use rendering::object::ToGlName;
use rendering::shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage};
use rendering::texture::{Filter, Ranged, Sampling, Texture, Texture2D, TextureMode, Wrap, RGBA};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem::{size_of, ManuallyDrop};
use std::ptr::null;
use vek::Clamp;

// The texel that will be stored within the main eGUI texture
type Texel = RGBA<Ranged<u8>>;

// Convert some image data into the RGBA texels
fn image_data_to_texels(image: &ImageData) -> Vec<Texel> {
    match image {
        // I don't like this but I have to cope
        ImageData::Color(color) => unsafe {
            color
                .pixels
                .iter()
                .map(|pixel| {
                    let vec = vek::Vec4::new(pixel.r(), pixel.g(), pixel.b(), pixel.a());
                    let ranged = vec.map(Ranged);
                    RGBA(ranged)
                })
                .collect::<Vec<Texel>>()
        },

        // Iterate through each alpha pixel and create a full color from it
        ImageData::Alpha(alpha) => {
            let mut texels = Vec::<Texel>::with_capacity(alpha.pixels.len() * 4);
            for alpha in alpha.pixels.iter() {
                texels.push(RGBA(vek::Vec4::broadcast(Ranged(*alpha))));
            }
            texels
        }
    }
}

// A global painter that will draw the eGUI elements onto the screen canvas
pub struct Painter {
    // A simple 2D shader that will draw the shapes
    shader: Shader,

    // Main font texture
    texture: Option<Texture2D<Texel>>,

    // The VAO for the whole painter mesh
    vao: u32,

    // Dynamic buffers that we will update each frame
    indices: ElementBuffer<u32>,
    vertices: ArrayBuffer<egui::epaint::Vertex>,
}

impl Painter {
    // Create a new painter using an asset loader an OpenGL context
    pub(super) fn new(loader: &mut AssetLoader, ctx: &mut Context) -> Self {
        // Load the shader stages first, then compile a shader
        let vert = loader.load::<VertexStage>("defaults/shaders/gui/vert.vrsh.glsl").unwrap();
        let frag = loader.load::<FragmentStage>("defaults/shaders/gui/frag.frsh.glsl").unwrap();

        // Link the stages and compile the shader
        let shader = ShaderCompiler::link((vert, frag), Processor::from(loader), ctx);

        // Create the main mesh VAO
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // Resizable buffers for vertices and indices
        let vertices = ArrayBuffer::<egui::epaint::Vertex>::new(ctx, BufferMode::Resizable, &[]).unwrap();
        let indices = ElementBuffer::<u32>::new(ctx, BufferMode::Resizable, &[]).unwrap();

        // Set the vertex attribute parameters for the position, uv, and color attributes
        unsafe {
            const STRIDE: i32 = size_of::<egui::epaint::Vertex>() as i32;
            gl::BindBuffer(gl::ARRAY_BUFFER, vertices.name());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, STRIDE, null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, STRIDE, (size_of::<f32>() * 2) as isize as _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::FALSE, STRIDE, (size_of::<f32>() * 4) as isize as _);
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
    pub fn draw(&mut self, device: &mut Device, ctx: &mut Context, meshes: Vec<ClippedMesh>, deltas: TexturesDelta) {
        // Update the main texture
        if let Some((tid, delta)) = deltas.set.iter().find(|(&tid, _)| tid == TextureId::Managed(0)) {
            // Insert the texture if we don't have it already
            self.texture.get_or_insert_with(|| {
                let dimensions = vek::Extent2::from_slice(&delta.image.size()).as_::<u16>();
                let texels = image_data_to_texels(&delta.image);
                Texture2D::new(ctx, TextureMode::Resizable, dimensions, Sampling::new(Filter::Nearest, Wrap::ClampToEdge), false, &texels).unwrap()
            });
        }

        // Setup shader uniforms
        let mut uniforms = self.shader.as_mut().uniforms();
        let sampler = self.texture.as_ref().unwrap().sampler();
        uniforms.set_sampler("u_sampler", sampler);

        // Setup OpenGL settings like blending settings and all
        let settings = RasterSettings {
            depth_test: None,
            scissor_test: None,
            primitive: PrimitiveMode::Triangles { cull: FaceCullMode::None },
            srgb: true,
            blend: Some(BlendMode::with(Factor::One, Factor::OneMinusSrcAlpha)),
        };

        // Bind the canvas and rasterize the meshes
        let mut rasterizer = device.canvas_mut().rasterizer(&mut self.shader, ctx);

        // Draw the meshes
        for mesh in meshes {
            // Update the buffers using data from the clipped mesh
            self.vertices.write(mesh.1.vertices.as_slice());
            self.indices.write(mesh.1.indices.as_slice());

            unsafe {
                rasterizer.draw_unchecked(self.vao, self.indices.name(), self.indices.len() as u32, &settings);
            }
        }
    }
}
