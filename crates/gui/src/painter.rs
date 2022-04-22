use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use crate::buffers::Buffers;
use crate::common::{convert_image, get_dimensions, get_id};
use egui::TexturesDelta;
use egui::{epaint::Mesh, ClippedMesh, Rect};
use nohash_hasher::NoHashHasher;
use rendering::basics::texture::{ResizableTexture, Texture2D, TextureFlags, TextureParams, TextureLayout, TextureFormat, TextureFilter};
use rendering::basics::uniforms::Uniforms;
use rendering::gl;
use rendering::utils::DataType;
use rendering::{
    basics::{
        shader::{Shader, ShaderInitSettings},
        texture::TextureWrapMode,
    },
    pipeline::{Handle, Pipeline},
};
use vek::Clamp;

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: Handle<Shader>,

    // Multiple textures
    pub(crate) textures: HashMap<u64, Handle<Texture2D>, BuildHasherDefault<NoHashHasher<u64>>>,
    pub(crate) buffers: Buffers,
}

impl Painter {
    // Create a new painter
    pub fn new(pipeline: &mut Pipeline) -> Self {
        // Load the GUI shader
        let shader_settings = ShaderInitSettings::default()
            .source("defaults/shaders/gui/vert.vrsh.glsl")
            .source("defaults/shaders/gui/frag.frsh.glsl");
        let shader = Shader::new(shader_settings).unwrap();
        let shader = pipeline.insert(shader);
        Self {
            shader,
            textures: Default::default(),
            buffers: Buffers::new(pipeline),
        }
    }
    // Set uniforms
    fn set_mesh_uniforms(&mut self, mesh: &Mesh, uniforms: &mut Uniforms, last_texture: &mut Handle<Texture2D>) {
        // Get ID
        let id = match mesh.texture_id {
            egui::TextureId::Managed(id) => id,
            egui::TextureId::User(_) => todo!(),
        };

        let handle = self.textures.get(&id).unwrap();
        // Only set the uniform if we need to
        if handle != last_texture {
            uniforms.set_texture2d("u_sampler", handle);
            *last_texture = handle.clone();
        }
    }
    // Draw a single egui mesh
    fn draw_mesh(&mut self, rect: Rect, mesh: Mesh, pipeline: &Pipeline) {
        // We already have the shader bound, so we just need to draw
        // Get the rect size so we can use the scissor test
        let pixels_per_point = pipeline.window().pixels_per_point() as f32;
        let clip_min = vek::Vec2::new(pixels_per_point * rect.min.x, pixels_per_point * rect.min.y);
        let clip_max = vek::Vec2::new(pixels_per_point * rect.max.x, pixels_per_point * rect.max.y);
        let dims = pipeline.window().dimensions().as_().into();
        let clip_min = clip_min.clamped(vek::Vec2::zero(), dims);
        let clip_max = clip_max.clamped(vek::Vec2::zero(), dims);
        let clip_min: vek::Vec2<i32> = clip_min.round().as_();
        let clip_max: vek::Vec2<i32> = clip_max.round().as_();

        //scissor Y coordinate is from the bottom
        unsafe {
            gl::Scissor(clip_min.x, dims.y as i32 - clip_max.y, clip_max.x - clip_min.x, clip_max.y - clip_min.y);
        }

        // Gotta fil the buffers with new data, then we can draw
        self.buffers.fill_buffers(mesh.vertices, mesh.indices);
        self.buffers.draw();
    }
    // Apply the texture deltas
    fn apply_deltas(&mut self, pipeline: &mut Pipeline, deltas: TexturesDelta) {
        // Create / modify
        for (tid, delta) in deltas.set {
            if let Some(handle) = self.textures.get(&get_id(tid)) {
                // Simply update the texture
                let texture = pipeline.get_mut(handle).unwrap();
                if delta.is_whole() {
                    // Resize and write
                    texture.resize_then_write(get_dimensions(&delta.image), convert_image(delta.image));
                }
            } else {
                // If we don't have the texture ID stored, we must create it
                let texture = Texture2D::new(
                    get_dimensions(&delta.image),
                    Some(convert_image(delta.image)),
                    TextureParams {
                        wrap: TextureWrapMode::ClampToEdge(None),
                        flags: TextureFlags::RESIZABLE,
                        layout: TextureLayout::new(DataType::U8, TextureFormat::RGBA8R),
                        filter: TextureFilter::Linear,
                    },
                );
                // Create the texture handle
                let texture = pipeline.insert(texture);
                self.textures.insert(get_id(tid), texture);
            }
        }
        // Delete
        for tid in deltas.free {
            // Dropping the handle will automatically get rid of the texture
            self.textures.remove(&get_id(tid)).unwrap();
        }
    }
    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline, clipped_meshes: Vec<ClippedMesh>, deltas: TexturesDelta) {
        // No need to draw if we don't have any meshes or if our shader is invalid
        if clipped_meshes.is_empty() || pipeline.get(&self.shader).is_none() {
            return;
        }

        // Apply the texture deltas
        self.apply_deltas(pipeline, deltas);

        // Since all the elements use the same shader, we can simply set it once
        let shader = pipeline.get(&self.shader).unwrap();

        // OpenGL settings
        unsafe {
            // UI is rendered after the scene is rendered, so it is fine to bind to the default framebuffer since we are going to use it to render the screen quad anyways
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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
    }
}
