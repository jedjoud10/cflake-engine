use std::ptr::null_mut;

use ahash::AHashMap;

use super::{
    introspect, ComputeShader, ComputeStage, FragmentStage, Processor, Program, Shader, Stage,
    VertexStage, Block, BlockIndex,
};
use crate::context::{Context, ToGlName};

// Compile a shader program using multiple unlinked shader stages
unsafe fn compile(username: String, names: &[u32], ctx: &mut Context) -> Program {
    // Create the program and link the stages to it
    let program = gl::CreateProgram();
    for name in names {
        gl::AttachShader(program, *name);
    }

    // Link the stages together to finalize the shader
    gl::LinkProgram(program);

    // Check for shader linking validity
    let mut result = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut result);
    if result == 0 {
        // Create a string that will contain the error message
        let mut len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let message = String::from_utf8({
            let mut vec = vec![0; len as usize + 1];
            gl::GetProgramInfoLog(program, len, null_mut(), vec.as_mut_ptr() as _);
            vec
        })
        .unwrap();

        // Print the error message
        panic!("Error: \n{}", message);
    }

    // Use shader introspection to pre-fetch the shader uniform/storage blocks and uniform locations
    let introspection = introspect(program);

    // Fetch all the uniform locations
    let uniform_locations: AHashMap<String, u32> = introspection
        .uniforms()
        .iter()
        .map(|uniform| (uniform.name().to_string(), uniform.location()))
        .collect();

    // Fetch all the buffer indices
    let buffer_block_locations: AHashMap<String, BlockIndex> = introspection
        .blocks()
        .iter()
        .map(|block| (block.name().to_string(), *block.index()))
        .collect();

    Program {
        username,
        name: program,
        introspection,
        buffer_block_locations,
        uniform_locations,
        _phantom: Default::default(),
    }
}

// This trait will be implemented for valid combinations of multiple unique stages
pub trait StageSet {
    // The output shader type (either a normal shader or a compute shader)
    type OutShaderType;

    // Link the multiple inputs into the valid shader
    unsafe fn link(input: Self, processor: Processor, ctx: &mut Context) -> Self::OutShaderType;
}

impl StageSet for (VertexStage, FragmentStage) {
    type OutShaderType = Shader;

    unsafe fn link(
        input: Self,
        mut processor: Processor,
        ctx: &mut Context,
    ) -> Self::OutShaderType {
        // Process shader directives and includes
        let username = format!("{}-{}", input.0.name(), input.1.name());
        let vertex = processor.filter(input.0);
        let fragment = processor.filter(input.1);

        // Compile the stages
        let vertex = super::stage::compile(ctx, vertex);
        let fragment = super::stage::compile(ctx, fragment);

        // And compile the main shader
        Shader(compile(username, &[vertex.name(), fragment.name()], ctx))
    }
}

impl StageSet for ComputeStage {
    type OutShaderType = ComputeShader;

    unsafe fn link(
        input: Self,
        mut processor: Processor,
        ctx: &mut Context,
    ) -> Self::OutShaderType {
        // Process shader directives and includes
        let username = input.name().to_string();
        let compute = processor.filter(input);

        // Compile the single stage
        let compute = super::stage::compile(ctx, compute);

        // And compile the main shader
        ComputeShader(compile(username, &[compute.name()], ctx))
    }
}

// A shader linker/compiler will take multiple shader sources and combine them to create a specific shader (this is simply a wrapper around StageSet btw)
pub struct ShaderCompiler;
impl ShaderCompiler {
    // Simply link multiple shader stages into a shader
    pub fn link<C: StageSet>(
        input: C,
        processor: Processor,
        ctx: &mut Context,
    ) -> C::OutShaderType {
        unsafe { C::link(input, processor, ctx) }
    }
}
