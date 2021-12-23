use others::SmartList;
use std::{collections::{HashMap, HashSet}, sync::{mpsc::Sender, Mutex}};

use crate::{GPUObject, GPUObjectID, MainThreadMessage, ModelGPUObject, MaterialGPUObject, SubShaderGPUObject, ShaderGPUObject, ComputeShaderGPUObject, TextureGPUObject, TextureFillGPUObject, RendererGPUObject};

// A simple Buffer containing the GPU objects that have been generated on the pipeline thread
#[derive(Default)]
pub struct PipelineBuffer {
    pub gpuobjects: SmartList<GPUObject>,                               // GPU objects
    pub waitable_objects: HashMap<u64, usize>,                          // Waitable ID to GPUObject index
    pub callback_objects: HashMap<u64, (usize, std::thread::ThreadId)>, // Callback ID to GPUObject index
    pub names_to_id: HashMap<String, usize>,                            // Names to GPUObject index

    // This is for tasks that have executed but that do not have a an associated GPU object
    pub executed_tasks: HashSet<u64>, // Executed tasks
}

impl PipelineBuffer {
    // Send messages to the main thread telling it what callbacks we must execute
    pub fn execute_callbacks(&mut self, tx2: &Sender<MainThreadMessage>) {
        // Send a message to the main thread saying what callbacks we must run
        let callbacks_objects_indices = std::mem::take(&mut self.callback_objects);
        let callback_objects = callbacks_objects_indices
        .into_iter()
        .map(|(callback_id, (index, thread_id))| {
            (
                callback_id,
                (self.get_gpuobject_usize(&index).unwrap().clone(), GPUObjectID { index: Some(index) }),
                thread_id,
            )
        })
        .collect::<Vec<(u64, (GPUObject, GPUObjectID), std::thread::ThreadId)>>();
        
        // Now we must all of this to the main thread
        for (callback_id, args, thread_id) in callback_objects {
            tx2.send(MainThreadMessage::ExecuteCallback(callback_id, args, thread_id)).unwrap();
        }
    }
    // Add a GPU object to the buffer, returning it's GPUObjectID
    pub fn add_gpuobject(&mut self, gpuobject: GPUObject, name: Option<String>) -> GPUObjectID {
        // Insert the gpu object
        let index = self.gpuobjects.add_element(gpuobject);
        // If we have a name add it
        match name {
            Some(name) => {
                self.names_to_id.insert(name, index);
                let x = GLOBAL_BUFFER.lock().unwrap();
                x.names_to_id.insert(name, index);
            }
            None => {}
        }
        GPUObjectID { index: Some(index) }
    }
    // Remove a GPU object from the buffer
    pub fn remove_gpuobject(&mut self, id: GPUObjectID) {
        self.gpuobjects.remove_element(id.index.unwrap()).unwrap();
    }
    // Add some additional data like callback ID or waitable ID to the GPU object
    pub fn received_new_gpuobject_additional(&mut self, id: GPUObjectID, callback_id: Option<(u64, std::thread::ThreadId)>, waitable_id: Option<u64>) {
        let index = id.index.unwrap();
        match callback_id {
            Some((id, thread_id)) => {
                self.callback_objects.insert(id, (index, thread_id));
            }
            None => { /* We cannot run a callback on this object */ }
        }
        match waitable_id {
            Some(id) => {
                self.waitable_objects.insert(id, index);
            }
            None => { /* We cannot run the un-wait function on the threads awaiting this object */ }
        }
    }
    // We have received confirmation that we have executed a specific task
    pub fn received_task_execution_ack(&mut self, execution_id: u64) {
        // Add the GPU object to the current interface buffer
        self.executed_tasks.insert(execution_id);
    }    
    // Get a GPU using it's name
    pub fn get_named_gpuobject(&self, name: &str) -> Option<GPUObject> {
        let index = self.names_to_id.get(name);
        match index {
            Some(index) => {
                let gpuobject = self.gpuobjects.get_element(*index);
                // Flatten
                gpuobject.flatten().cloned()
            }
            None => None,
        }
    }
    // Get a GPU object using it's GPUObjectID
    pub fn get_gpuobject(&self, id: &GPUObjectID) -> Option<&GPUObject> {
        self.gpuobjects.get_element(id.index?)?
    }
    // Get a mutable GPU object using it's GPUObjectId
    pub fn get_gpuobject_mut(&mut self, id: &GPUObjectID) -> Option<&mut GPUObject> {
        self.gpuobjects.get_element_mut(id.index?)?
    }
    // Get a GPU object using a ref usize
    pub fn get_gpuobject_usize(&self, index: &usize) -> Option<&GPUObject> {
        self.gpuobjects.get_element(*index)?
    }
    // Get multiple GPU object using a ref usize
    pub fn get_gpu_object_usize_batch<'a, I>(&self, ids: I) -> Option<Vec<&GPUObject>> 
    where
        I: Iterator<Item = &'a usize>{
        let mut gpuobjects = ids.map(|x| self.gpuobjects.get_element(*x).flatten().unwrap()).collect::<Vec<&GPUObject>>();
        Some(gpuobjects)
    }
}

// Conversions
impl PipelineBuffer {
    pub fn as_model(&self, id: &GPUObjectID) -> Option<&ModelGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Model(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_material(&self, id: &GPUObjectID) -> Option<&MaterialGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Material(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_subshader(&self, id: &GPUObjectID) -> Option<&SubShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::SubShader(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_shader(&self, id: &GPUObjectID) -> Option<&ShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Shader(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_compute_shader(&self, id: &GPUObjectID) -> Option<&ComputeShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::ComputeShader(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_texture(&self, id: &GPUObjectID) -> Option<&TextureGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Texture(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_texture_fill(&self, id: &GPUObjectID) -> Option<&TextureFillGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::TextureFill(x) = object {
            Some(x)
        } else { None }
    }
    pub fn as_renderer(&self, id: &GPUObjectID) -> Option<&RendererGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Renderer(x) = object {
            Some(x)
        } else { None }
    }
}

use lazy_static::lazy_static;
lazy_static! {
   pub(crate) static ref GLOBAL_BUFFER: Mutex<GlobalBuffer> = Mutex::new(GlobalBuffer::default());
}

// A global buffer that will be accessible by each worker thread
#[derive(Default)]
pub struct GlobalBuffer {
    pub names_to_id: HashMap<String, usize>,
}

impl GlobalBuffer {
    // Get if a GPU object name is present in the global buffer
    pub fn gpuobject_name_valid(&self, name: &str) -> bool {
        self.names_to_id.contains_key(name)
    }
    // Get the ID of a GPU object name from within the buffer
    pub fn get_id(&self, name: &str) -> Option<GPUObjectID> {
        let index = self.names_to_id.get(name)?;
        Some(GPUObjectID { index: Some(*index) })
    }
}