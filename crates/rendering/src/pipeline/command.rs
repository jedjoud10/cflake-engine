use std::sync::{Arc, Mutex};

use crate::{GPUObject, Model, Renderer, Shader, SubShader, Texture, TextureGPUObject, TextureType};

// A shared GPU object that was sent to the render thread, and that can be returned back to the main thread at some point
pub struct SharedData<T: Default> {
    pub object: Arc<T>,
}

impl<T> SharedData<T>
where
    T: Default,
{
    pub fn new(x: T) -> Self {
        Self { object: Arc::new(x) }
    }
}

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
    DisposeRenderer(usize),
    // Update the transform of a specific renderer
    UpdateRendererTransform(),
    // Shader stuff
    SubShaderCreate(SharedData<SubShader>),
    ShaderCreate(SharedData<Shader>),
    // Textures
    TextureCreate(SharedData<Texture>),
    TextureCreateNull(SharedData<Texture>),
    TextureUpdateSize(TextureGPUObject, TextureType),
    TextureUpdateData(TextureGPUObject, Vec<u8>),
    TextureFillArrayVeclib(TextureGPUObject),
    TextureFillArray(TextureGPUObject),
    // Model
    ModelCreate(SharedData<Model>),
    // Compute
    ComputeRun(),
    ComputeLock(),
    // Window settings
    WindowSizeUpdate(u16, u16, f32),
    // Destroy the render thread, since we are exiting from the application
    DestroyRenderPipeline(),
}
