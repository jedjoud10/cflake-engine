use std::sync::Arc;

use crate::{GPUObject, Model, Renderer, Shader, SubShader, Texture};

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
}
// A render task (A specific message passed to the render thread)
pub enum RenderTask {
    // Renderers
    AddRenderer(usize, Arc<Renderer>),
    DisposeRenderer(usize),
    // Shader stuff
    CreateSubShader(SubShader),
    CreateShader(Shader),
    GenerateTexture(Texture),

    RefreshModel(Model),
    RunCompute(),
    // Destroy the render thread, since we are exiting from the application
    DestroyRenderPipeline(),
}
