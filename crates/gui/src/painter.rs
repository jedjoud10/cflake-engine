use std::sync::Arc;

use egui::{epaint::Mesh, ClippedMesh, Color32, FontImage, Output, Rect};
use rendering::{
    basics::{
        shader::{Shader, ShaderInitSettings, ShaderInitSettingsBuilder},
        texture::{Texture, TextureFilter, TextureFormat, TextureDimensions, TextureWrapping},
        uniforms::Uniforms,
    },
    pipeline::{Pipeline, Handle},
    utils::DataType,
};

use crate::buffers::Buffers;

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: Handle<Shader>,
    pub(crate) egui_font_texture: Handle<Texture>,
    pub(crate) egui_font_image_arc: FontImage,
    pub(crate) egui_font_texture_version: Option<u64>,
    pub(crate) clipped_meshes: Vec<ClippedMesh>,
    pub(crate) buffers: Buffers,
    pub(crate) output: Output,
}

impl Painter {
    // Create a new painter
    // This will be called on the render thread
    pub fn new(pipeline: &mut Pipeline) -> Self {
        // Load the GUI shader
        let shader_settings = ShaderInitSettingsBuilder::default()
            .source("defaults/shaders/gui/vert.vrsh.glsl")
            .source("defaults/shaders/gui/frag.frsh.glsl")
            .build();
        let shader = Shader::new(shader_settings).unwrap();
        let shader = pipeline.shaders.insert(shader);
        // Load the egui font texture
        let egui_font_texture = Texture::default()
            .with_filter(TextureFilter::Linear)
            .with_format(TextureFormat::RGBA8R)
            .with_wrapping_mode(TextureWrapping::ClampToEdge)
            .with_data_type(DataType::U8)
            .with_mipmaps(false);
        let egui_font_texture = pipec::construct(pipeline, egui_font_texture).unwrap();
        Self {
            shader,
            egui_font_texture,
            egui_font_image_arc: Default::default(),
            clipped_meshes: Default::default(),
            output: Default::default(),
            buffers: Buffers::new(),
            egui_font_texture_version: Default::default(),
        }
    }
    // Draw a single egui mesh
    fn draw_mesh(&mut self, rect: Rect, mesh: Mesh, pipeline: &mut Pipeline, pixels_per_point: f32) {
        // We already have the shader bound, so we just need to draw
        // Get the rect size so we can use the scissor test
        let clip_min = veclib::vec2(pixels_per_point * rect.min.x, pixels_per_point * rect.min.y);
        let clip_max = veclib::vec2(pixels_per_point * rect.max.x, pixels_per_point * rect.max.y);
        let clip_min = clip_min.clamp(veclib::Vector2::ZERO, pipeline.window.dimensions.into());
        let clip_max = clip_max.clamp(veclib::Vector2::ZERO, pipeline.window.dimensions.into());
        let clip_min: veclib::Vector2<i32> = clip_min.round().into();
        let clip_max: veclib::Vector2<i32> = clip_max.round().into();

        //scissor Y coordinate is from the bottom
        unsafe {
            gl::Scissor(
                clip_min.x,
                pipeline.window.dimensions.y as i32 - clip_max.y,
                clip_max.x - clip_min.x,
                clip_max.y - clip_min.y,
            );
        }

        // Gotta fil the buffers with new data, then we can draw
        self.buffers.fill_buffers(mesh.vertices, mesh.indices);
        self.buffers.draw();
    }
    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline) {
        // No need to draw if we don't have any meshes or if our shader is invalid
        if self.clipped_meshes.is_empty() || pipeline.shaders.get(self.shader).is_none() {
            return;
        }
        // Le texture
        self.upload_egui_font_texture(pipeline);

        // Since all the elements use the same shader, we can simply set it once
        let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.shader));
        let uniforms = Uniforms::using_mut_pipeline(&settings, pipeline);
        // For now, the single texture we can draw is the font texture. We won't be able to set user textures, but that is an upcoming feature
        uniforms.set_texture("u_sampler", self.egui_font_texture, 0);

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

        let clipped_meshes = std::mem::take(&mut self.clipped_meshes);
        for ClippedMesh(rect, mesh) in clipped_meshes {
            self.draw_mesh(rect, mesh, pipeline, pipeline.window.pixels_per_point as f32);
        }

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
    // Upload the egui font texture and update it's OpenGL counterpart if it changed
    fn upload_egui_font_texture(&mut self, pipeline: &mut Pipeline) {
        // Only update if we need to
        let texture = self.egui_font_image_arc.as_ref();
        if Some(texture.version) == self.egui_font_texture_version {
            return;
        }
        // I hate this
        let mut bytes = Vec::<u8>::with_capacity(texture.pixels.len() * 4);
        for alpha in texture.pixels.iter() {
            let color = Color32::from_white_alpha(*alpha);
            bytes.push(color.r());
            bytes.push(color.g());
            bytes.push(color.b());
            bytes.push(color.a());
        }

        // Update the OpenGL version
        let gl_tex = pipeline.textures.get_mut(self.egui_font_texture);
        if let Some(gl_tex) = gl_tex {
            let dimensions = TextureDimensions::Texture2D(texture.width as u16, texture.height as u16);
            gl_tex.update_size_fill(dimensions, bytes).unwrap();
            // Don't forget to update the version
            self.egui_font_texture_version = Some(texture.version);
        } else {
            self.egui_font_texture_version = None;
        }
    }
}
