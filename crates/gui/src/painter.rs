use std::sync::{atomic::AtomicBool, Arc};

use egui::{epaint::ClippedShape, ClippedMesh, CtxRef, Output};
use rendering::{
    basics::{
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureFormat}, uniforms::{ShaderUniformsSettings, ShaderIDType, Uniforms},
    },
    object::ObjectID,
    pipeline::{pipec, Pipeline},
};

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: ObjectID<Shader>,
    pub(crate) egui_font_texture: ObjectID<Texture>,
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
        let egui_font_texture = Texture::default().with_filter(TextureFilter::Linear).with_format(TextureFormat::RGBA8R).with_mipmaps(false);
        let egui_font_texture = pipec::construct(pipeline, egui_font_texture).unwrap();
        Self {
            shader,
            egui_font_texture,
            clipped_meshes: Default::default(),
            output: Default::default(),
            font_image: Default::default(),
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
}
