use std::{num::NonZeroU32, ptr::null_mut};

use super::{ComputeShader, ComputeStage, FragmentStage, Processor, Shader, VertexStage, Program};
use crate::context::{Context, ToGlName};

// Compile a shader program using multiple unlinked shader stages 
unsafe fn compile(names: &[NonZeroU32]) -> Program {
    // Create the program and link the stages to it
    let program = gl::CreateProgram();
    for name in names {
        gl::AttachShader(program, name.get());
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
            let mut vec = Vec::with_capacity(len as usize + 1);
            gl::GetProgramInfoLog(program, len, null_mut(), vec.spare_capacity_mut().as_mut_ptr() as _);
            vec
        }).unwrap();

        // Print the error message
        panic!("Error: \n{}", message);
    }

    // Delete the shader stages since we already linked them
    for name in names {
        gl::DeleteShader(name.get());
    }

    // Return the program GL name
    //NonZeroU32::new(program).unwrap()
    todo!()
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

    unsafe fn link(input: Self, mut processor: Processor, ctx: &mut Context) -> Self::OutShaderType {
        // Process shader directives and includes
        let vertex = processor.filter(input.0);
        let fragment = processor.filter(input.1);

        // Compile the stages
        let vertex = super::stage::compile(ctx, vertex);
        let fragment = super::stage::compile(ctx, fragment);

        // And compile the main shader
        Shader(compile(&[vertex.name(), fragment.name()]))
    }
}

impl StageSet for ComputeStage {
    type OutShaderType = ComputeShader;

    unsafe fn link(input: Self, mut processor: Processor, ctx: &mut Context) -> Self::OutShaderType {
        // Process shader directives and includes
        let compute = processor.filter(input);
        
        // Compile the single stage
        let compute = super::stage::compile(ctx, compute);
    
        // And compile the main shader
        ComputeShader(compile(&[compute.name()]))
    }
}

// A shader linker will take multiple shader sources and combine them to create a specific shader (this is simply a wrapper around StageSet btw)
pub struct StageLinker;
impl StageLinker {
    // Simply link multiple shader stages into a shader
    pub fn link<C: StageSet>(input: C, processor: Processor, ctx: &mut Context) -> C::OutShaderType {
        unsafe { C::link(input, processor, ctx) }
    }
}
