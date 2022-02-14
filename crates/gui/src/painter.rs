use std::sync::{atomic::AtomicBool, Arc};

use egui::{CtxRef, ClippedMesh, epaint::ClippedShape, Output};
use rendering::{pipeline::{Pipeline, pipec}, basics::{shader::{Shader, ShaderSettings}, texture::Texture}, object::ObjectID};

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
        // Load the 
        Self {
            shader,
            egui_font_texture,
            clipped_meshes: Default::default(),
            output: Default::default(),
            font_image: Default::default()
        }
    }
    // Draw a single egui mesh
    fn draw_mesh(&self, mesh: ClippedMesh, shader: &Shader) {
    }

    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline) {
        // No need to draw if we don't have any meshes 
        if self.clipped_meshes.is_empty() { return; }       
        let shader = pipeline.shaders.get(self.shader);
        dbg!("Draw");
        
        // We might not have a valid shader yet, so we make sure it is valid
        if let Some(shader) = shader {
            // Now we can peacefully draw
            let clipped_meshes = std::mem::take(&mut self.clipped_meshes);
            for mesh in clipped_meshes {
                self.draw_mesh(mesh, shader);
            }
        }
    }
}