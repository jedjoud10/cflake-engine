use gl::types::GLuint;
use std::{ffi::c_void, ptr::null};

use crate::{
    basics::{
        readwrite::ReadBytes,
        shader::{
            info::{QueryParameter, QueryResource::ShaderStorageBlock, Resource, ShaderInfoQuerySettings},
            query_shader_info,
        },
        transfer::Transfer,
        uniforms::ShaderIDType,
    },
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID, PipelineObject},
    pipeline::Pipeline,
    utils::{AccessType, UpdateFrequency, UsageType},
};

// Some custom fetching info just for shader storage blocks
struct CustomFetcher {
    // Certifier shader moment
    shader: ShaderIDType,

    // The block's name
    name: String,

    // Multiplier value since arrays with no constant length act like they have a length of 1 when their byte size is fetched
    mul: usize,
}

// An OpenGL SSBO
pub struct ShaderStorage {
    // The OpenGL name for the underlying buffer
    pub(crate) oid: GLuint,
    // How we access the shader storage
    pub usage: UsageType,
    // Some default data
    pub(crate) bytes: Vec<u8>,
    // The size in bytes of the underlying data
    pub(crate) byte_size: usize,

    // Custom SSBO block fetcher that we can use to allocate the SSBO with the perfect amount of bytes
    fetcher: Option<CustomFetcher>,
}
impl PipelineObject for ShaderStorage {
    // Reserve an ID for this shader storage
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.shader_storages.gen_id()))
    }
    // Send this shader storage to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::ShaderStorage(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::ShaderStorage(Deconstruct::<Self>(id))
    }
    // Add the shader storage to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // If we are using an SSBO block fetcher, we gotta fetch the byte size
        if let Some(fetcher) = self.fetcher.take() {
            // Fetch
            let program = fetcher.shader.get_program(pipeline);

            // Create some query settings
            let mut settings = ShaderInfoQuerySettings::default();
            let resource = Resource {
                res: ShaderStorageBlock,
                name: fetcher.name,
            };
            settings.query(resource.clone(), vec![QueryParameter::ByteSize]);

            // Query
            let shader_info = query_shader_info(program, settings);

            // Read back the byte size
            let byte_size = shader_info.get(&resource).unwrap().get(0).unwrap().as_byte_size().unwrap();
            self.byte_size = *byte_size * fetcher.mul;
        }

        // Create the SSBO
        unsafe {
            gl::GenBuffers(1, &mut self.oid);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
            // Get the default data if we need to
            let data_ptr = if !self.bytes.is_empty() {
                self.bytes.as_ptr() as *const c_void
            } else {
                null() as *const c_void
            };
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, self.byte_size as isize, data_ptr, self.usage.convert());
        }
        // Add the shader storage
        pipeline.shader_storages.insert(id, self);
        Some(())
    }
    // Remove the compute shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.shader_storages.remove(id)
    }
}
impl ShaderStorage {
    // Create a new empty shader storage
    pub fn new(frequency: UpdateFrequency, access: AccessType, byte_size: usize) -> Self {
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: Default::default(),
            byte_size,
            fetcher: Default::default(),
        }
    }
    // Create a new empty shader storage that will fetch a specific shader storage block from a shader, and initialize it's size using that
    pub fn new_using_block(frequency: UpdateFrequency, access: AccessType, shader: ShaderIDType, block_name: &str, mul: usize) -> Self {
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: Default::default(),
            byte_size: 0,
            fetcher: Some(CustomFetcher {
                shader,
                name: block_name.to_string(),
                mul,
            }),
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
            fetcher: Default::default(),
        }
    }
    // Read some bytes from the SSBO
    pub(crate) fn read_bytes(&self, pipeline: &Pipeline, read: Transfer<ReadBytes>) -> GlTracker {
        GlTracker::new(
            move |_pipeline| unsafe {
                // Bind the buffer before reading
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                // If we have a range, we can use it
                let range = read.0.range;
                let bytes = if let Some(range) = range {
                    // Read using specific range
                    let offset = range.start;
                    let size = range.end - range.start;
                    // Since we use a range, make a vector that can only hold that range
                    let mut vec = vec![0; size as usize];
                    gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, offset as isize, size as isize, vec.as_mut_ptr() as *mut c_void);
                    vec
                } else {
                    // Read the whole buffer
                    let mut vec = vec![0; self.byte_size as usize];
                    gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, 0, self.byte_size as isize, vec.as_mut_ptr() as *mut c_void);
                    vec
                };
                // Now store the shader storage's bytes
                let mut output_bytes = read.0.bytes.lock();
                *output_bytes = bytes;
            },
            pipeline,
        )
    }
}
