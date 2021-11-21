use std::sync::{Arc, Mutex};

use crate::{GPUObject, Model, ModelGPUObject, Renderer, RendererGPUObject, Shader, ShaderUniformsGroup, SubShader, Texture, TextureGPUObject, TextureType};

// A shared GPU object that was sent to the render thread, and that can be returned back to the main thread at some point
pub struct SharedData<T: Default + Sync> {
    pub object: Arc<T>,
}

impl<T> SharedData<T>
where
    T: Default + Sync,
{
    pub fn new(x: T) -> Self {
        Self { object: Arc::new(x) }
    }
}

pub enum RenderTaskReturn {
    GPUObject(GPUObject),
    TextureFillData(Vec<u8>),
}

pub enum RenderTaskStatus {
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
    // Shader stuff
    SubShaderCreate(SharedData<SubShader>),
    ShaderCreate(SharedData<Shader>),
    ShaderUniformGroup(SharedData<ShaderUniformsGroup>),
    // Textures
    TextureCreate(SharedData<Texture>),
    TextureUpdateSize(TextureGPUObject, TextureType),
    TextureUpdateData(TextureGPUObject, Vec<u8>),
    TextureFillArray(TextureGPUObject),
    TextureFillArrayVeclib(TextureGPUObject),
    // Model
    ModelCreate(SharedData<Model>),
    ModelDispose(ModelGPUObject),
    // Compute
    ComputeRun(),
    ComputeLock(),
    // Renderer
    RendererAdd(SharedData<Renderer>),
    RendererRemove(RendererGPUObject),
    RendererUpdateTransform(SharedData<math::utils::Transform>),
    // Window settings
    WindowSizeUpdate(u16, u16, f32),
    // Pipeline
    DestroyRenderPipeline(),
}
