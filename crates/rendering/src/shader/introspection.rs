use std::{ptr::null_mut, mem::MaybeUninit, ffi::CString};

use crate::object::ToGlName;

// The type of block that we have stored
pub enum Index {
    UniformBlock(u32),
    ShaderStorageBlock(u32),
}

// A single block, can represent a uniform block or an SSBO block
pub struct Block {
    // Full name of this block
    name: String,

    // Index for this specific block
    index: Index,

    // Byte size of this block
    size: usize,
}

// A single uniform value stored within the shader
pub struct Uniform {
    // Full name of the uniform
    name: String,

    // Location for this uniform
    location: u32,
}

impl Uniform {
    // Get the uniform's name
    pub fn name(&self) -> &str {
        &self.name
    }

    // Get the uniform's location
    pub fn location(&self) -> u32 {
        self.location
    }
}

// Shader introspection is how we can fetch the shader block binding points and such
pub struct Introspection {
    // All the blocks that are stored within the introspection data
    blocks: Vec<Block>,

    // All the uniform variables that the shader program uses
    uniforms: Vec<Uniform>,
}

impl Introspection {
    // Get all the blocks that were fetched
    pub fn blocks(&self) -> &[Block] {
        self.blocks.as_slice()
    }

    // Get all the uniforms that were fetched
    pub fn uniforms(&self) -> &[Uniform] {
        self.uniforms.as_slice()
    }
}

// Raw block properties given directly from opengl
#[repr(C)]
struct BlockProps {
    // The length of the name of the block
    name_length: i32,

    // The current buffer binding. Very useful for uniform blocks
    buffer_binding: i32,

    // The current data size, in bytes
    data_size: i32,
}

// Raw uniform properties given directly from opengl
#[repr(C)]
struct UnifProps {
    // The length of the name of the uniform
    name_length: i32,

    // The location of the uniform
    location: i32,

    // The block index of the uniform (tells us if it is from within a block or not)
    block_index: i32,

    // The OpenGL type of the uniform
    gltype: i32,
}

// Fetch the uniform properties of a unique uniform using it's index
unsafe fn unif_props(program: u32, index: u32) -> UnifProps {
    let props = [gl::NAME_LENGTH, gl::LOCATION, gl::BLOCK_INDEX, gl::TYPE];
    let mut output = MaybeUninit::uninit();
    let ptr: *mut i32 = std::mem::transmute(output.as_mut_ptr() as *mut UnifProps);
    gl::GetProgramResourceiv(
        program,
        gl::UNIFORM,
        index,
        props.len() as i32,
        &props as *const _,
        props.len() as i32,
        null_mut(),
        ptr,
    );
    output.assume_init()
}

// Fetch the block properties of a unique block using it's index
unsafe fn block_props(program: u32, interface: u32, index: u32) -> BlockProps {    
    let props = [gl::NAME_LENGTH, gl::BUFFER_BINDING, gl::BUFFER_DATA_SIZE];
    let mut output = MaybeUninit::uninit();
    let ptr: *mut i32 = std::mem::transmute(output.as_mut_ptr() as *mut BlockProps);
    gl::GetProgramResourceiv(
        program,
        interface,
        index,
        props.len() as i32,
        &props as *const _,
        props.len() as i32,
        null_mut(),
        ptr,
    );
    output.assume_init()
}
// Get the name of an arbitrary value using it's properties
unsafe fn name(program: u32, name_length: i32, interface: u32, index: u32) -> String {
    // Then fetch the name
    let mut name = vec![0u8; name_length as usize];
    gl::GetProgramResourceName(
        program,
        interface,
        index,
        name_length,
        null_mut(),
        name.as_mut_ptr() as *mut i8,
    );

    // Return a valid string
    CString::from_vec_with_nul(name).unwrap().into_string().unwrap()
}

// Get a vector of shader blocks directly from the program
unsafe fn fetch_blocks(program: u32, interface: u32, max: i32) -> Vec<Block> {
    (0..max)
        .into_iter()
        .map(|i| {
            // Read the block properties, and decompose it's values
            let props = block_props(program, interface, i as u32);
            let size = props.data_size as usize;
            let name = name(program, props.name_length, interface, i as u32);
            
            // Classify the index into the valid enum variant
            let index = if interface == gl::UNIFORM_BLOCK {
                Index::UniformBlock(i as u32)
            } else if interface == gl::SHADER_STORAGE_BLOCK {
                Index::ShaderStorageBlock(i as u32)
            } else {
                panic!()
            };

            // Construct block and add it to the vector
            Block { name, index, size }
        })
        .collect::<Vec<_>>()
}

// Get a vector of shader uniforms directly from the program (this will only fetch uniforms that are not contained within uniform blocks)
unsafe fn fetch_uniforms(program: u32, max: i32) -> Vec<Uniform> {
    (0..max)
        .into_iter()
        .map(|i| {
            // Read the uniform properties, and decompose it's values
            let props = unif_props(program, i as u32);
            let name = name(program, props.name_length, gl::UNIFORM, i as u32);
            
            // Construct the uniform and add it to the vector
            Uniform { name, location: props.location as u32, }
        })
        .collect::<Vec<_>>()
}

// Introspect a shader, and construct an Introspection struct
pub(super) unsafe fn introspect(program: u32) -> Introspection {
    // Count the number of uniform blocks and shader storage blocks
    let mut uniform_blocks = 0;
    let mut storage_blocks = 0;
    gl::GetProgramInterfaceiv(
        program,
        gl::UNIFORM_BLOCK,
        gl::ACTIVE_RESOURCES,
        &mut uniform_blocks,
    );
    gl::GetProgramInterfaceiv(
        program,
        gl::SHADER_STORAGE_BLOCK,
        gl::ACTIVE_RESOURCES,
        &mut storage_blocks,
    );

    // Count the number of non-block uniforms
    let mut non_block_uniforms = 0;
    gl::GetProgramInterfaceiv(
        program,
        gl::UNIFORM,
        gl::ACTIVE_RESOURCES,
        &mut non_block_uniforms,
    );

    // Fetch the valid block types and add them to a single vector
    let mut blocks = fetch_blocks(program, gl::UNIFORM_BLOCK, uniform_blocks);
    blocks.extend(fetch_blocks(program, gl::SHADER_STORAGE_BLOCK, storage_blocks));

    // Fetch uniform variables and store them within a single vector
    let uniforms = fetch_uniforms(program, non_block_uniforms);

    // TODO: Handle block uniform variables

    Introspection { blocks, uniforms }
}
