use crate::buffers::Buffers;
use crate::common::{convert_image, get_dimensions, get_id};
use assets::loader::AssetLoader;
use egui::TexturesDelta;
use egui::{epaint::Mesh, ClippedMesh, Rect};
use nohash_hasher::NoHashHasher;
use rendering::context::Context;
use rendering::gl;
use rendering::shader::{FragmentStage, Processor, Shader, ShaderCompiler, Uniforms, VertexStage};
use rendering::texture::{Ranged, Texture, Texture2D, RGBA};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use vek::Clamp;

// The texels that will be stored within the main egui texture
type Texel = RGBA<Ranged<u8>>;

// This will get a clip rectangle that we will use for OpenGL scissor tests
fn clip_rect(rect: Rect, ctx: &mut Context) -> (vek::Vec2<i32>, vek::Extent2<i32>) {
    /*
    let pixels_per_point = ctx.window().pixels_per_point() as f32;
    let clip_min = vek::Vec2::new(pixels_per_point * rect.min.x, pixels_per_point * rect.min.y);
    let clip_max = vek::Vec2::new(pixels_per_point * rect.max.x, pixels_per_point * rect.max.y);
    let dims = ctx.window().dimensions().as_().into();
    let clip_min = clip_min.clamped(vek::Vec2::zero(), dims);
    let clip_max = clip_max.clamped(vek::Vec2::zero(), dims);
    let clip_min: vek::Vec2<i32> = clip_min.round().as_();
    let clip_max: vek::Vec2<i32> = clip_max.round().as_();
    */
    todo!()
}

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // A simple 2D shader that will draw the shapes
    shader: Shader,

    // Main egui texture ID, and the OpenGL texture
    texture: Option<(u64, Texture2D<Texel>)>,

    // Raw OpenGL buffers
    buffers: Buffers,
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
            buffers: Buffers::new(ctx),
        }
    }

    // Draw a single egui mesh onto the screen
    fn draw(&mut self, clip: (vek::Vec2<i32>, vek::Extent2<i32>), mesh: Mesh, ctx: &mut Context) {
        /*
        //scissor Y coordinate is from the bottom
        unsafe {
            gl::Scissor(clip_min.x, dims.y as i32 - clip_max.y, clip_max.x - clip_min.x, clip_max.y - clip_min.y);
        }

        // Gotta fil the buffers with new data, then we can draw
        self.buffers.fill_buffers(mesh.vertices, mesh.indices);
        self.buffers.draw();
        */
    }

    // Draw the whole graphical user interface onto the screen.
    pub fn draw_gui(&mut self, ctx: &mut Context, meshes: Vec<ClippedMesh>, deltas: TexturesDelta) {
        // No meshes, no drawing. Ez
        if meshes.is_empty() {
            return;
        }

        // Use the default framebuffer for drawing
        let def = ctx.framebuffers().main();
        // Assuming that we only have one texture to deal with
        let main = &self.texture.unwrap().1;
        uniforms.set_sampler("u_sampler", main.sampler());
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
