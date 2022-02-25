use gl::types::GLuint;
use std::{ffi::c_void, ptr::null};

use crate::{
    basics::{
        buffer_operation::BufferOperation,
        shader::{
            info::{
                QueryParameter, QueryResource::ShaderStorageBlock, Resource,
                ShaderInfoQuerySettings,
            },
            query_shader_info,
        },
        uniforms::ShaderIDType,
    },
    object::{
        Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID,
        PipelineObject,
    },
    pipeline::Pipeline,
    utils::UsageType,
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
    // Is this shader storage dynamic?
    pub dynamic: bool,
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
            let byte_size = shader_info
                .get(&resource)
                .unwrap()
                .get(0)
                .unwrap()
                .as_byte_size()
                .unwrap();

            self.byte_size = byte_size.next_power_of_two() * fetcher.mul;
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
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                self.byte_size as isize,
                data_ptr,
                self.usage.convert(),
            );
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
    pub fn new(usage: UsageType, byte_size: usize) -> Self {
        Self {
            oid: 0,
            usage,
            bytes: Default::default(),
            byte_size,
            fetcher: Default::default(),
            dynamic: false,
        }
    }
    // Create a new empty shader storage that will fetch a specific shader storage block from a shader, and initialize it's size using that
    pub fn new_using_block(
        usage: UsageType,
        shader: ShaderIDType,
        block_name: &str,
        mul: usize,
    ) -> Self {
        Self {
            oid: 0,
            usage,
            bytes: Default::default(),
            byte_size: 0,
            fetcher: Some(CustomFetcher {
                shader,
                name: block_name.to_string(),
                mul,
            }),
            dynamic: false,
        }
    }
    // Create a new shader storage with some default data
    // Type T must have a repr(C) layout
    pub fn new_default<T: Sized>(usage: UsageType, default: T, byte_size: usize) -> Self {
        let borrow = &default;
        let slice =
            unsafe { std::slice::from_raw_parts::<u8>(borrow as *const T as *const u8, byte_size) };
        Self {
            oid: 0,
            usage,
            bytes: slice.to_vec(),
            byte_size,
            fetcher: Default::default(),
            dynamic: false,
        }
    }
    // Create a new dynamic shader storage
    pub fn new_dynamic(usage: UsageType) -> Self {
        Self {
            oid: 0,
            usage,
            bytes: Default::default(),
            byte_size: 0,
            fetcher: Default::default(),
            dynamic: true,
        }
    }
    // Read/Write some bytes from/to the SSBO
    pub(crate) fn buffer_operation(&mut self, op: BufferOperation) -> GlTracker {
        match op {
            BufferOperation::Write(mut write) => {
                GlTracker::fake(|| unsafe {
                    // Bind the buffer before writing
                    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                    write.bytes.shrink_to_fit();
                    // If the given data contains more bytes than what we can handle, we must re-allocate the buffer and increase it's size
                    if write.bytes.len() > self.byte_size {
                        if self.dynamic {
                            // Reallocate
                            gl::BufferData(
                                gl::SHADER_STORAGE_BUFFER,
                                write.bytes.len() as isize,
                                write.bytes.as_ptr() as *const c_void,
                                self.usage.convert(),
                            );
                            self.byte_size = write.bytes.len();
                            eprintln!("Writing to SSBO with {} bytes", write.bytes.len());
                        } else {
                            panic!("Buffer is not dynamic, cannot reallocate!");
                        }
                    } else {
                        // We have enough bytes allocated already
                        gl::BufferSubData(
                            gl::SHADER_STORAGE_BUFFER,
                            0,
                            write.bytes.len() as isize,
                            write.bytes.as_ptr() as *const c_void,
                        );
                    }
                })
            }
            BufferOperation::Read(read) => {
                GlTracker::new(|| unsafe {
                    // Bind the buffer before reading
                    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                    // Read the whole buffer
                    let mut vec = vec![0u8; self.byte_size as usize];
                    gl::GetBufferSubData(
                        gl::SHADER_STORAGE_BUFFER,
                        0,
                        self.byte_size as isize,
                        vec.as_mut_ptr() as *mut c_void,
                    );
                    // Now store the shader storage's bytes
                    let mut output_bytes = read.bytes.lock();
                    *output_bytes = vec;
                })
            }
        }
    }
}
