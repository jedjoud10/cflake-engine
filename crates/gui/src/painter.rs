use assets::loader::AssetLoader;
use egui::TexturesDelta;
use egui::{epaint::Mesh, ClippedMesh, Rect};
use nohash_hasher::NoHashHasher;
use rendering::buffer::{ArrayBuffer, ElementBuffer};
use rendering::context::{Context, Device};
use rendering::gl;
use rendering::shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage};
use rendering::texture::{Ranged, Texture, Texture2D, RGBA};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use vek::Clamp;

// The texels that will be stored within the main egui texture
type Texel = RGBA<Ranged<u8>>;

// This will get a clip rectangle that we will use for OpenGL scissor tests
fn clip_rect(rect: Rect, device: &Device) -> (vek::Vec2<i32>, vek::Extent2<i32>) {
    // Convert the eGUi positions into vek positions
    let min = vek::Vec2::<f32>::from(<(f32, f32)>::from(rect.min));
    let max = vek::Vec2::<f32>::from(<(f32, f32)>::from(rect.max));

    // Convert the extents to pixels
    let ppo = device.window().scale_factor() as f32;
    let min = min * ppo;
    let max = max * ppo;

    // Clamp to the window size and round to the nearest pixel
    let size = vek::Vec2::<f32>::from(device.size().as_::<f32>());
    let min = min.clamped(vek::Vec2::zero(), size);
    let max = max.clamped(vek::Vec2::zero(), size);

    // Le rounding and casting
    let min = min.round().as_::<i32>();
    let max = max.round().as_::<i32>();

    // Convert the min/max bounds to a rectange
    (min, vek::Extent2::<i32>::from(max - min))
}

// A global painter that will draw the eGUI elements onto the screen canvas
pub struct Painter {
    // A simple 2D shader that will draw the shapes
    shader: Shader,

    // Main egui texture ID, and the OpenGL texture
    texture: Option<(u64, Texture2D<Texel>)>,

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

        Self {
            shader,
            texture: None,
            vao: todo!(),
            indices: todo!(),
            vertices: todo!(),
        }
    }

    // Draw the whole user interface onto the screen
    pub fn draw_gui(&mut self, device: &mut Device, ctx: &mut Context, meshes: Vec<ClippedMesh>, deltas: TexturesDelta) {
        // Update the main texture

        // Setup shader uniforms and bind canvas

        // Setup OpenGL settings like blending settings and all

        // Setup Scissor and disable depth

        /*
        // Use the default framebuffer for drawing
        let def = ctx.framebuffers().main();
        // Assuming that we only have one texture to deal with
        let main = &self.texture.unwrap().1;
        uniforms.set_sampler("u_sampler", main.sampler());
        */
        /*
        // Apply the texture deltas
        self.apply_deltas(pipeline, deltas);

        // Since all the elements use the same shader, we can simply set it once
        let shader = pipeline.get(&self.shader).unwrap();

        // UI is rendered after the scene is rendered, so it is fine to bind to the default framebuffer since we are going to use it to render the screen quad anyways
        renderer.framebuffer_mut().bind(|mut bound| {
            // OpenGL settings
            unsafe {
                gl::BindVertexArray(self.buffers.vao);
                gl::Enable(gl::FRAMEBUFFER_SRGB);
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
                gl::Disable(gl::CULL_FACE);
                gl::Disable(gl::DEPTH_TEST);
                gl::Enable(gl::SCISSOR_TEST);
            }

            // We can bind once, and mutate multiple times
            Uniforms::new(shader.program(), pipeline, |mut uniforms| {
                // Draw each mesh
                let mut last_texture = Handle::<Texture2D>::default();
                for ClippedMesh(rect, mesh) in clipped_meshes {
                    self.set_mesh_uniforms(&mesh, &mut uniforms, &mut last_texture);
                    self.draw_mesh(rect, mesh, pipeline);
                }
            });

            // Reset
            unsafe {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
                gl::BindVertexArray(0);
                gl::Disable(gl::BLEND);
                gl::Enable(gl::CULL_FACE);
                gl::Enable(gl::DEPTH_TEST);
                gl::Disable(gl::SCISSOR_TEST);
            }
        });
        */
    }
}
