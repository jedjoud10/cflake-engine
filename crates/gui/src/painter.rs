use std::sync::{atomic::AtomicBool, Arc};

use egui::{epaint::{ClippedShape, Mesh}, ClippedMesh, Color32, CtxRef, Output, FontImage, Rect};
use rendering::{
    basics::{
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureFormat, TextureType},
        uniforms::{ShaderIDType, ShaderUniformsSettings, Uniforms},
    },
    object::ObjectID,
    pipeline::{pipec, Pipeline},
    utils::DataType,
};

use crate::buffers::Buffers;

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: ObjectID<Shader>,
    pub(crate) egui_font_texture: ObjectID<Texture>,
    pub(crate) egui_font_image_arc: Arc<FontImage>,
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
        let shader_settings = ShaderSettings::default()
            .source("defaults\\shaders\\gui\\vert.vrsh.glsl")
            .source("defaults\\shaders\\gui\\frag.frsh.glsl");
        let shader = Shader::new(shader_settings).unwrap();
        let shader = pipec::construct(pipeline, shader).unwrap();
        // Load the egui font texture
        let egui_font_texture = Texture::default()
            .with_filter(TextureFilter::Linear)
            .with_format(TextureFormat::RGBA8R)
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
    fn draw_mesh(&self, rect: Rect, mesh: Mesh, pipeline: &mut Pipeline, pixel_per_point: f64) {
        // We already have the shader bound, so we just need to draw
    }
    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline) {
        // No need to draw if we don't have any meshes
        if self.clipped_meshes.is_empty() {
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
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
        }

        let clipped_meshes = std::mem::take(&mut self.clipped_meshes);
        for ClippedMesh(rect, mesh) in clipped_meshes {
            self.draw_mesh(rect, mesh, pipeline, pipeline.window.pixel_per_point);
        }

        // Reset
        unsafe {
            gl::Disable(gl::FRAMEBUFFER_SRGB);
            gl::Disable(gl::BLEND);
            gl::Enable(gl::CULL_FACE);
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
        let gl_tex = pipeline.textures.get_mut(self.egui_font_texture).unwrap();
        let dimensions = TextureType::Texture2D(texture.width as u16, texture.height as u16);
        gl_tex.update_size_fill(dimensions, bytes).unwrap();
        // Don't forget to update the version
        self.egui_font_texture_version = Some(texture.version);
    }
}
