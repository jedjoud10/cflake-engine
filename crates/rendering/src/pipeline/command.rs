use std::sync::Arc;

use crate::{GPUObject, Model, Renderer, Shader, Texture};


// Render task status
pub enum RenderTaskStatus {
    PendingStartup,
    Succsessful(Option<GPUObject>),
    Failed,
}

// A render command
pub struct RenderCommand {
    // Message stuff
    pub message_id: u128,
    pub input_task: RenderTask,
    pub status: RenderTaskStatus,
}
// A render task (A specific message passed to the render thread)
pub enum RenderTask {
    // Basic render stuff like rendering entities
    AddRenderer(usize, Arc<Renderer>),
    DisposeRenderer(usize),
    CreateShader(Arc<Shader>), // Give it the shader source
    GenerateTexture(Arc<Texture>),
    RefreshModel(Arc<Model>),
    RunCompute(),
    // Destroy the render thread, since we are exiting from the application
    DestroyRenderPipeline()
}