use crate::basics::*;
use std::sync::mpsc::{Receiver, Sender};

// A render command
pub struct RenderCommand {
    // Message stuff
    pub message_id: usize,
    pub task: RenderTask,
}

// The output of a specific render command
pub struct RenderCommandReturn {
    pub message_id: usize,
}

// A render task (A specific message passed to the render thread)
pub enum RenderTask {
    // Basic render stuff like rendering entities
    AddRenderer(Renderer),
    DisposeRenderer(Renderer),
    // Shaders
    CreateShader(Shader),
    // Textures
    GenerateTexture(Texture),
    // Models
    RefreshModel(Model),
    // Compute shaders
    RunCompute(),
}

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
pub struct RenderPipeline {
    // Channel stuff
    pub rx: std::sync::mpsc::Receiver<RenderCommand>,
    pub tx: std::sync::mpsc::Sender<RenderCommand>,
}

impl RenderPipeline {
    // Create the new render thread
    pub fn initialize_render_thread(&mut self) {
        let (_tx, _rx): (Sender<Model>, Receiver<Model>) = std::sync::mpsc::channel();
    }
}
