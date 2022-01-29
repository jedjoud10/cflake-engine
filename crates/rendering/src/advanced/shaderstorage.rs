use crate::{
    basics::Buildable,
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    utils::{AccessType, UpdateFrequency, UsageType},
};

// An OpenGL SSBO
pub struct ShaderStorage {
    // The OpenGL name for the underlying buffer
    pub(crate) oid: u32,
    // How we access the shader storage
    pub usage: UsageType,
    // Some default data
    pub(crate) bytes: Vec<u8>,
    // The size in bytes of the underlying data
    pub(crate) byte_size: usize,
}

impl PipelineObject for ShaderStorage {}

impl Buildable for ShaderStorage {
    fn construct_task(self, pipeline: &crate::pipeline::Pipeline) -> (crate::object::PipelineTask, crate::object::ObjectID<Self>) {
        // Create the ID
        let id = pipeline.atomics.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create a task and send it
        (PipelineTask::CreateShaderStorage(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

impl ShaderStorage {
    // Create a new empty shader storage
    pub fn new<T: Sized>(frequency: UpdateFrequency, access: AccessType, byte_size: usize) -> Self {
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: Vec::new(),
            byte_size,
        }
    }
    // Create a new shader storage with some default data
    // Type T must have a repr(C) layout
    pub fn new_default<T: Sized>(frequency: UpdateFrequency, access: AccessType, default: T, byte_size: usize) -> Self {
        let borrow = &default;
        let slice = unsafe { std::slice::from_raw_parts::<u8>(borrow as *const T as *const u8, byte_size) };
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: slice.to_vec(),
            byte_size,
        }
    }
}
