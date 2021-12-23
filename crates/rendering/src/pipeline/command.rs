use crate::{
    interface, internal_task, ComputeShaderGPUObject, GPUObject, GPUObjectID, Material, Model, ModelGPUObject, Renderer, RendererGPUObject, Shader, ShaderUniformsGroup, SubShader,
    Texture, TextureGPUObject, TextureType, IS_RENDER_THREAD,
};
use lazy_static::lazy_static;
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

lazy_static! {
    // Counter that increments for Callback ID, Waitable ID, and Execution ID
    static ref COUNTER: AtomicU64 = AtomicU64::new(0);
}

// A shared GPU object that was sent to the render thread
pub struct SharedData<T: Default + Send> {
    object: T,
}

impl<T> SharedData<T>
where
    T: Default + Send,
{
    pub fn new(x: T) -> Self {
        Self { object: x }
    }
    pub fn get(self) -> T {
        self.object
    }
}

thread_local! {
    // The render task sender!
    pub static RENDER_COMMAND_SENDER: RefCell<Option<std::sync::mpsc::Sender<crate::RenderCommandQuery>>> = RefCell::new(None);
}

// Send a render command to the render thread
fn command(command: RenderCommandQuery) {
    // Create the render command
    // No need to check if we are on the render thread, because even if we are on the render thread, we can't do anything differently
    // Send the command query
    crate::RENDER_COMMAND_SENDER.with(|x| {
        let sender_ = x.borrow();
        let sender = sender_.as_ref().unwrap();
        // Send the command to the thread
        sender.send(command).unwrap();
    });
}

// The immediate result for a render command
pub struct RenderCommandResult {
    task: Option<RenderTask>,
}

impl RenderCommandResult {
    // Create a new query result from a specific render command
    pub fn new(task: RenderTask) -> Self {
        Self { task: Some(task) }
    }
    // Explicitly tell this command query result to send the result immediatly
    pub fn send(mut self) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = RenderCommandQuery {
            task,
            callback_id: None,
            waitable_id: None,
            execution_id: None,
            thread_id: std::thread::current().id(),
        };
        command(query);
    }
    // Set callback for this specific command query result. It will receive a notif from the main thread when to execute this callback
    pub fn with_callback(mut self, callback_id: u64) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = RenderCommandQuery {
            task,
            callback_id: Some(callback_id),
            waitable_id: None,
            execution_id: None,
            thread_id: std::thread::current().id(),
        };
        command(query);
    }
    // Simply wait for this command to be executed
    pub fn wait(mut self) {
        if !IS_RENDER_THREAD.with(|x| x.get()) {
            // Send the command, but with a special command ID that we must wait for
            let execution_id = COUNTER.fetch_add(1, Ordering::Relaxed);
            todo!()
        } else {
            panic!();
        }
    }
    // We will wait for the result of this render command query
    pub fn wait_gpuobject(mut self) -> GPUObject {
        if !IS_RENDER_THREAD.with(|x| x.get()) {
            // Send the command, but with a special command ID that we must wait for
            let waitable_id = COUNTER.fetch_add(1, Ordering::Relaxed);
            todo!()
        } else {
            panic!();
        }
    }
    // We will wait for thes result of this render command query as a GPUObject ID
    pub fn wait_gpuobject_id(mut self) -> GPUObjectID {
        if !IS_RENDER_THREAD.with(|x| x.get()) {
            // Send the command, but with a special command ID that we must wait for
            let waitable_id = COUNTER.fetch_add(1, Ordering::Relaxed);
            todo!()
        } else {
            panic!();
        }
    }
}

impl std::ops::Drop for RenderCommandResult {
    // Custom drop function that actually sends the command, just in case where we did not explicitly do that
    fn drop(&mut self) {
        // Send the command
        match self.task.take() {
            Some(task) => {
                let query = RenderCommandQuery {
                    task,
                    callback_id: None,
                    waitable_id: None,
                    execution_id: None,
                    thread_id: std::thread::current().id(),
                };
                command(query);
            }
            None => { /* We have called a function that invalidates the task */ }
        }
    }
}

// A render command query
pub struct RenderCommandQuery {
    pub callback_id: Option<u64>,
    pub waitable_id: Option<u64>,
    pub execution_id: Option<u64>,
    pub thread_id: std::thread::ThreadId,
    pub task: RenderTask,
}
// A render task (A specific message passed to the render thread)
pub enum RenderTask {
    // Shader stuff
    SubShaderCreate(SharedData<SubShader>),
    ShaderCreate(SharedData<Shader>),
    ShaderUniformGroup(SharedData<ShaderUniformsGroup>),
    // Textures
    TextureCreate(SharedData<Texture>),
    TextureUpdateSize(GPUObjectID, TextureType),
    TextureUpdateData(GPUObjectID, Vec<u8>),
    TextureFillArray(GPUObjectID, usize),
    // Model
    ModelCreate(SharedData<Model>),
    ModelDispose(GPUObjectID),
    // Compute
    ComputeRun(ComputeShaderGPUObject, (u16, u16, u16), ShaderUniformsGroup),
    ComputeLock(ComputeShaderGPUObject),
    // Renderer
    RendererAdd(SharedData<(Renderer, veclib::Matrix4x4<f32>)>),
    RendererRemove(GPUObjectID),
    RendererUpdateTransform(GPUObjectID, SharedData<veclib::Matrix4x4<f32>>),
    // Material
    MaterialCreate(SharedData<Material>),
    MaterialUpdateUniforms(GPUObjectID, SharedData<ShaderUniformsGroup>),
    // Window settings
    WindowUpdateSize(veclib::Vector2<u16>),
    WindowUpdateVSync(bool),
    WindowUpdateFullscreen(bool),
    // Pipeline
    CameraDataUpdate(SharedData<(veclib::Vector3<f32>, veclib::Quaternion<f32>, veclib::Vector2<f32>, veclib::Matrix4x4<f32>)>),
}
