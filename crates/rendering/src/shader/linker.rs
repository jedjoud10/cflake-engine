use super::{ComputeShader, ComputeStage, FragmentStage, Processor, Shader, VertexStage};
use crate::context::Context;

// This trait will be implemented for valid combinations of multiple unique stages
pub trait StageSet {
    // The output shader type (either a normal shader or a compute shader)
    type OutShaderType;

    // Link the multiple inputs into the valid shader
    fn link(input: Self, processor: Processor, ctx: &mut Context) -> Self::OutShaderType;
}

impl StageSet for (VertexStage, FragmentStage) {
    type OutShaderType = Shader;

    fn link(input: Self, mut processor: Processor, ctx: &mut Context) -> Self::OutShaderType {
        let vertex = processor.filter(input.0);
        let fragment = processor.filter(input.1);
        todo!()
    }
}

impl StageSet for ComputeStage {
    type OutShaderType = ComputeShader;

    fn link(input: Self, mut processor: Processor, ctx: &mut Context) -> Self::OutShaderType {
        let vertex = processor.filter(input);
        todo!()
    }
}

// A shader linker will take multiple shader sources and combine them to create a specific shader (this is simply a wrapper around StageSet btw)
pub struct StageLinker;
impl StageLinker {
    // Simply link multiple shader stages into a shader
    pub fn link<C: StageSet>(input: C, processor: Processor, ctx: &mut Context) -> C::OutShaderType {
        C::link(input, processor, ctx)
    }
}
