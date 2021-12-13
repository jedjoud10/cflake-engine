use std::sync::Arc;

use crate::{
    ComputeShaderGPUObject, GPUObject, Model, ModelGPUObject, Renderer, RendererGPUObject, Shader, ShaderUniformsGroup, SubShader, Texture, TextureGPUObject, TextureType,
};

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

// The return type of the render task
pub enum RenderTaskReturn {
    None, // Oopsie woopsie! I did a wittle fuckie wuckie >w<. Please excwuse my shitty code. Tehe!
    GPUObject(GPUObject, String),
}

// A render command
pub struct RenderCommand {
    pub name: String,
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
    TextureFillArray(TextureGPUObject, usize),
    // Model
    ModelCreate(SharedData<Model>),
    ModelDispose(ModelGPUObject),
    // Compute
    ComputeRun(ComputeShaderGPUObject, (u16, u16, u16), ShaderUniformsGroup),
    ComputeLock(ComputeShaderGPUObject),
    // Renderer
    RendererAdd(SharedData<(Renderer, veclib::Matrix4x4<f32>)>),
    RendererRemove(usize),
    RendererUpdateTransform(RendererGPUObject, SharedData<(veclib::Vector3<f32>, veclib::Quaternion<f32>, veclib::Vector3<f32>)>),
    // Window settings
    WindowUpdateSize(veclib::Vector2<u16>),
    WindowUpdateVSync(bool),
    WindowUpdateFullscreen(bool),
    // Pipeline
    DestroyRenderThread,
    CameraDataUpdate(SharedData<(veclib::Vector3<f32>, veclib::Quaternion<f32>, veclib::Vector2<f32>, veclib::Matrix4x4<f32>)>),
}
