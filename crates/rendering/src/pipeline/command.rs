use crate::{
    internal_task, ComputeShaderGPUObject, GPUObject, GPUObjectID, Material, Model, ModelGPUObject, Renderer, RendererGPUObject, Shader, ShaderUniformsGroup, SubShader,
    Texture, TextureGPUObject, TextureType, is_render_thread,
};
use lazy_static::lazy_static;
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicU64, Ordering, AtomicPtr, AtomicBool},
        Arc, Mutex,
    },
};
use super::buffer::PipelineBuffer;

lazy_static! {
    static ref COMMAND_ID: AtomicU64 = AtomicU64::new(0);
}

fn increment_command_id() -> u64 {
    COMMAND_ID.fetch_add(1, Ordering::Relaxed)
}

// A shared GPU object that was sent to the render thread
pub struct SharedData<T: Send> {
    object: T,
}

impl<T> SharedData<T>
where
    T: Send,
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
pub struct RenderCommandQueryResult {
    task: Option<RenderTask>, // The task that we will send to the render thread OR that we will execute internally    
    id: Option<GPUObjectID> // In case that we have loaded a GPU object ID already
}

impl RenderCommandQueryResult {
    // Create a new query result from a specific render command
    pub fn new(task: RenderTask) -> Self {
        Self { task: Some(task), id: None }
    }
    // Create a new query result from a loaded ID
    pub fn new_id(id: GPUObjectID) -> Self {
        Self { task: None, id: Some(id) }
    }
    // Explicitly tell this command query result to send the result immediatly
    pub fn send(mut self) {
        if is_render_thread() { panic!() }
        // Send the command
        let task = self.task.take().unwrap();
        let command_id = increment_command_id();
        let query = RenderCommandQuery {
            task,
            callback_id: None,
            command_id,
            thread_id: std::thread::current().id(),
        };
        command(query);
    }
    // Set callback for this specific command query result. It will receive a notif from the main thread when to execute this callback
    pub fn with_callback(mut self, callback_id: u64) {
        if is_render_thread() { panic!() }
        // Send the command
        let task = self.task.take().unwrap();
        let command_id = increment_command_id();
        let query = RenderCommandQuery {
            task,
            callback_id: Some(callback_id),
            command_id: command_id,
            thread_id: std::thread::current().id(),
        };
        command(query);
    }
    // We will wait for thes result of this render command query as a GPUObject ID
    pub fn wait(mut self) -> GPUObjectID {
        // Panic if we are on the render thread
        if is_render_thread() { panic!() }
        // Send the command, and wait for it's return value
        match self.id {
            Some(x) => x,
            None => {
                let command_id = increment_command_id();
                let task = self.task.take().unwrap();
                let query = RenderCommandQuery {
                    task,
                    callback_id: None,
                    command_id,
                    thread_id: std::thread::current().id(),
                };
                command(query);
                crate::others::wait_id(command_id)
            },
        }        
    }
    // Wait till we have executed the command on the render thread
    pub fn wait_execution(mut self) {
        // Panic if we are on the render thread
        if is_render_thread() { panic!() }
        // Send the command, and wait for it's return value
        let command_id = increment_command_id();
        let task = self.task.take().unwrap();
        let query = RenderCommandQuery {
            task,
            callback_id: None,
            command_id,
            thread_id: std::thread::current().id(),
        };
        command(query);
        crate::others::wait_execution(command_id)
    }

    // Wait for the creation of a GPU object, but internally
    pub fn wait_internal(mut self, buf: &mut PipelineBuffer) -> GPUObjectID {
        if !is_render_thread() { panic!() }
        match self.id {
            Some(x) => x,
            None => {
                // Execute the task
                let task = self.task.take().unwrap();
                let id = internal_task(buf, task);
                id.unwrap()
            }
        }        
    }
}

impl std::ops::Drop for RenderCommandQueryResult {
    // Custom drop function that actually sends the command, just in case where we did not explicitly do that
    fn drop(&mut self) {
        if !is_render_thread() {
            // Send the command
            match self.task.take() {
                Some(task) => {
                    let command_id = increment_command_id();
                    let query = RenderCommandQuery {
                        task,
                        callback_id: None,
                        command_id,
                        thread_id: std::thread::current().id(),
                    };
                    command(query);
                }
                None => { /* We have called a function that invalidates the task */ }
            }
        }        
    }
}

// A render command query
pub struct RenderCommandQuery {
    pub callback_id: Option<u64>,
    pub command_id: u64,
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
    TextureFillArray(GPUObjectID, usize, Arc<AtomicPtr<Vec<u8>>>),
    // Model
    ModelCreate(SharedData<Model>),
    ModelDispose(GPUObjectID),
    // Compute
    ComputeRun(GPUObjectID, (u16, u16, u16), ShaderUniformsGroup),
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
