use crate::{utils::{UsageType, UpdateFrequency, AccessType}, object::{PipelineObject, ObjectID, ConstructionTask, Construct}, pipeline::Pipeline};

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

impl PipelineObject for ShaderStorage {
    // Reserve an ID for this shader storage
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.shader_storages.get_next_id_increment())))
    }
    // Send this shader storage to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::ShaderStorage(Construct::<Self>(self, id))
    }
    // Add the shader storage to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the shader storage
        pipeline.shader_storages.insert(id.get()?, self);
        Some(())
    }
    // Remove the compute shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.shader_storages.remove(id.get()?)
    }
}
impl ShaderStorage {
    // Create a new empty shader storage
    pub fn new(frequency: UpdateFrequency, access: AccessType, byte_size: usize) -> Self {
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
