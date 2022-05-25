use std::{
    ffi::CString,
    num::NonZeroU32,
    ptr::{null, null_mut},
};

use crate::{context::Context, object::ToGlName};

use super::Program;

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

// Introspect a shader, and construct an Introspection struct
pub(super) unsafe fn introspect(program: u32) -> Introspection {
    // Count the number of uniform blocks and shader storage blocks
    let mut uniforms = 0;
    let mut storages = 0;
    gl::GetProgramInterfaceiv(program, gl::UNIFORM_BLOCK, gl::ACTIVE_RESOURCES, &mut uniforms);
    gl::GetProgramInterfaceiv(program, gl::SHADER_STORAGE_BLOCK, gl::ACTIVE_RESOURCES, &mut storages);

    // Raw block properties given directly from opengl
    #[repr(C)]
    #[derive(Default)]
    struct Props {
        // The length of the name of the block
        name_length: i32,

        // The current buffer binding. Very useful for uniform blocks
        buffer_binding: i32,

        // The current data size, in bytes
        data_size: i32,

        // These values will not be written to, so we are fineski
        program_interface: u32,
        index: u32,
        program: u32,
    }

    // This is a general function that will fetch some block properties
    unsafe fn props(program: u32, program_interface: u32, index: u32) -> Props {
        // Fetch the first 3 raw values, since they correspond to actual buffer properties
        let resprops = [gl::NAME_LENGTH, gl::BUFFER_BINDING, gl::BUFFER_DATA_SIZE];
        let mut output = Props::default();
        let ptr: *mut i32 = std::mem::transmute(&mut output as *mut Props);
        gl::GetProgramResourceiv(
            program,
            program_interface,
            index,
            resprops.len() as i32,
            &resprops as *const _,
            resprops.len() as i32,
            null_mut(),
            ptr,
        );

        // Overwrite the rest of the variables
        output.program_interface = program_interface;
        output.index = index;
        output.program = program;
        output
    }

    // This is a general function that will fetch the name of an arbitrary block
    unsafe fn name(props: &Props) -> String {
        // Then fetch the name
        let mut name = Vec::<u8>::with_capacity(props.name_length as usize);
        gl::GetProgramResourceName(
            props.program,
            props.program_interface,
            props.index,
            props.name_length,
            null_mut(),
            name.as_mut_ptr() as *mut i8,
        );

        // Return a valid string
        String::from_utf8(name).unwrap()
    }

    // Two functions that are no ops, but they simply make use of Props
    fn size(props: &Props) -> usize {
        props.data_size as usize
    }

    fn index(props: &Props) -> u32 {
        props.index as u32
    }

    // Get a vector of shader blocks directly from the program
    unsafe fn fetch_blocks(program: u32, program_interface: u32, max: i32) -> Vec<Block> {
        // Iterate through all the uniform block indices
        (0..max)
            .into_iter()
            .map(|i| {
                // Fetch block props
                let props = props(program, program_interface, i as u32);

                // And fetch unique values
                let name = name(&props);
                let index = index(&props);
                let size = size(&props);

                // Classify the index into the valid enum variant
                let index = if program_interface == gl::UNIFORM_BLOCK {
                    Index::UniformBlock(index)
                } else if program_interface == gl::SHADER_STORAGE_BLOCK {
                    Index::ShaderStorageBlock(index)
                } else {
                    panic!()
                };

                // Construct block and add it to the vector
                Block { name, index, size }
            })
            .collect::<Vec<_>>()
    }

    // Fetch the valid block types and add them to a single vector
    let mut blocks = fetch_blocks(program, gl::UNIFORM_BLOCK, uniforms);
    blocks.extend(fetch_blocks(program, gl::SHADER_STORAGE_BLOCK, storages));

    // Fetch uniform variables and store them within a single vector

    // Fetch the valid uniforms
    let uniforms = Default::default();

    Introspection { blocks, uniforms }
}
