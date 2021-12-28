use others::SmartList;
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::Sender, Mutex},
};

use crate::{
    ComputeShaderGPUObject, GPUObject, GPUObjectID, MainThreadMessage, MaterialGPUObject, ModelGPUObject, RendererGPUObject, ShaderGPUObject, SubShaderGPUObject, TextureGPUObject,
};

use super::async_command_data::AsyncGPUCommandData;

// A simple Buffer containing the GPU objects that have been generated on the pipeline thread
#[derive(Default)]
pub struct PipelineBuffer {
    pub gpuobjects: SmartList<GPUObject>,                                       // GPU objects
    pub callback_objects: HashMap<u64, (Option<usize>, std::thread::ThreadId)>, // Callback ID to option GPUObject index
    pub names_to_id: HashMap<String, usize>,                                    // Names to GPUObject index
    pub renderers: HashSet<GPUObjectID>,                                        // Renderers
    pub async_gpu_command_datas: Vec<AsyncGPUCommandData>,                      // Some sync data that will be polled each frame
    pub batch_commands_executed: HashMap<u64, u16>, // The number of commands that have executed, who where part of a batch. They are all linked to one u64, which is the batch callback ID
}

unsafe impl Send for PipelineBuffer {}

impl PipelineBuffer {
    // Send messages to the main thread telling it what callbacks we must execute
    pub fn execute_callbacks(&mut self, tx2: &Sender<MainThreadMessage>) {
        // Send a message to the main thread saying what callbacks we must run
        let callbacks_objects_indices = std::mem::take(&mut self.callback_objects);
        // Callbacks that return a valid GPU object
        let gpuobjects_callbacks = callbacks_objects_indices
            .iter()
            .filter_map(|(&callback_id, (index, thread_id))| match index {
                Some(index) => Some((
                    callback_id,
                    (self.get_gpuobject_usize(index).unwrap().clone(), GPUObjectID { index: Some(*index) }),
                    thread_id.clone(),
                )),
                None => None,
            })
            .collect::<Vec<(u64, (GPUObject, GPUObjectID), std::thread::ThreadId)>>();
        // Callbacks that are just here for executions
        let execution_callbacks = callbacks_objects_indices
            .iter()
            .filter_map(|(&callback_id, (index, thread_id))| match index {
                Some(_) => None,
                None => Some((callback_id, thread_id.clone())),
            })
            .collect::<Vec<(u64, std::thread::ThreadId)>>();

        // Now we must send all of this to the main thread
        for (callback_id, args, thread_id) in gpuobjects_callbacks {
            tx2.send(MainThreadMessage::ExecuteGPUObjectCallback(callback_id, args, thread_id)).unwrap();
        }
        for (callback_id, thread_id) in execution_callbacks {
            tx2.send(MainThreadMessage::ExecuteExecutionCallback(callback_id, thread_id)).unwrap();
        }
    }
    // Add a GPU object to the buffer, returning it's GPUObjectID
    pub fn add_gpuobject(&mut self, gpuobject: GPUObject, name: Option<String>) -> GPUObjectID {
        // Insert the gpu object
        let index = self.gpuobjects.add_element(gpuobject);
        let id = GPUObjectID { index: Some(index) };
        // If we have a name add it
        match name {
            Some(name) => {
                self.names_to_id.insert(name.clone(), index);
                crate::others::add_id(&name, id.clone());
            }
            None => {}
        }
        id
    }
    // Add the data for an async OpenGL command
    pub fn add_async_gpu_command_data(&mut self, x: AsyncGPUCommandData) {
        self.async_gpu_command_datas.push(x);
    }
    // Remove a GPU object from the buffer
    pub fn remove_gpuobject(&mut self, id: GPUObjectID) {
        self.gpuobjects.remove_element(id.index.unwrap()).unwrap();
    }
    // Add some additional data like callback ID or waitable ID to the GPU object
    pub fn received_new_gpuobject_additional(&mut self, gpuobject_id: Option<GPUObjectID>, callback_id: Option<(u64, std::thread::ThreadId)>) {
        match callback_id {
            Some((id, thread_id)) => {
                self.callback_objects.insert(id, (gpuobject_id.map(|x| x.index.unwrap()), thread_id));
            }
            None => { /* We cannot run a callback on this object */ }
        }
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
        I: Iterator<Item = &'a usize>,
    {
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
        } else {
            None
        }
    }
    pub fn as_material(&self, id: &GPUObjectID) -> Option<&MaterialGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Material(x) = object {
            Some(x)
        } else {
            None
        }
    }
    pub fn as_subshader(&self, id: &GPUObjectID) -> Option<&SubShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::SubShader(x) = object {
            Some(x)
        } else {
            None
        }
    }
    pub fn as_shader(&self, id: &GPUObjectID) -> Option<&ShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Shader(x) = object {
            Some(x)
        } else {
            None
        }
    }
    pub fn as_compute_shader(&self, id: &GPUObjectID) -> Option<&ComputeShaderGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::ComputeShader(x) = object {
            Some(x)
        } else {
            None
        }
    }
    pub fn as_texture(&self, id: &GPUObjectID) -> Option<&TextureGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Texture(x) = object {
            Some(x)
        } else {
            None
        }
    }
    pub fn as_renderer(&self, id: &GPUObjectID) -> Option<&RendererGPUObject> {
        let object = self.get_gpuobject(id)?;
        if let GPUObject::Renderer(x) = object {
            Some(x)
        } else {
            None
        }
    }
}
// The mut version
impl PipelineBuffer {
    pub fn as_renderer_mut(&mut self, id: &GPUObjectID) -> Option<&mut RendererGPUObject> {
        let object = self.get_gpuobject_mut(id)?;
        if let GPUObject::Renderer(x) = object {
            Some(x)
        } else {
            None
        }
    }
}
