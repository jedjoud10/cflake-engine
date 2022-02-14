use std::sync::{atomic::AtomicBool, Arc};

use egui::{epaint::ClippedShape, ClippedMesh, Color32, CtxRef, Output};
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

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: ObjectID<Shader>,
    pub(crate) egui_font_texture: ObjectID<Texture>,
    pub(crate) egui_font_texture_version: Option<u64>,
    pub(crate) egui_font_texture_bytes: Vec<u8>,
    pub(crate) clipped_meshes: Vec<ClippedMesh>,
    pub(crate) output: Output,
    pub(crate) font_image: Arc<egui::epaint::FontImage>,
}

impl Painter {
    // Create a new painter
    pub fn new(pipeline: &Pipeline) -> Self {
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
            clipped_meshes: Default::default(),
            output: Default::default(),
            font_image: Default::default(),
            egui_font_texture_version: Default::default(),
            egui_font_texture_bytes: Default::default(),
        }
    }
    // Draw a single egui mesh
    fn draw_mesh(&self, mesh: ClippedMesh, pipeline: &mut Pipeline, pixel_per_point: f64) {
        // We already have the shader bound, so we just need to draw
    }

    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline) {
        // No need to draw if we don't have any meshes
        if self.clipped_meshes.is_empty() {
            return;
        }
        // Since all the elements use the same shader, we can simply set it once
        let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.shader));
        let uniforms = Uniforms::using_mut_pipeline(&settings, pipeline);
        // Bind the texture and resolution
        uniforms.set_texture("u_sampler", self.egui_font_texture, 0);

        // Now we can peacefully draw
        let clipped_meshes = std::mem::take(&mut self.clipped_meshes);
        for mesh in clipped_meshes {
            self.draw_mesh(mesh, pipeline, pipeline.window.pixel_per_point);
        }
    }
    // Set the bytes for the egui font texture using the texture itself
    pub fn set_egui_font_texture_bytes(&mut self, texture: &egui::epaint::FontImage) {
        // Only update if we need to
        if Some(texture.version) == self.egui_font_texture_version {
            self.egui_font_texture_bytes.clear();
            return;
        } else
        // I hate this
        let mut bytes = Vec::<u8>::with_capacity(texture.pixels.len() * 4);
        for alpha in texture.pixels.iter() {
            let color = Color32::from_white_alpha(*alpha);
            bytes.push(color.r());
            bytes.push(color.g());
            bytes.push(color.b());
            bytes.push(color.a());
        }
        self.egui_font_texture_bytes = bytes;
    }
    // Upload a new egui font texture if we have some valid bytes that we can use
    pub fn upload_egui_font_texture_if_needed(&mut self, pipeline: &mut Pipeline) {
        

        // Update the OpenGL version
        let gl_tex = pipeline.textures.get_mut(self.egui_font_texture).unwrap();
        let dimensions = TextureType::Texture2D(texture.width as u16, texture.height as u16);
        gl_tex.update_size_fill(dimensions, bytes).unwrap();
        // Don't forget to update the version
        self.egui_font_texture_version = Some(texture.version);
    }
}
