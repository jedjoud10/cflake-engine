use crate::{Compiled, ComputeModule, FragmentModule, VertexModule, ReflectedModule, ReflectedShader};
use std::sync::Arc;

// A rendering shader that contains a vertex module and fragment module
// This shader might contain more modules though, like the tesselation modules
#[derive(Clone)]
pub struct Shader {
    vertex: Compiled<VertexModule>,
    fragment: Compiled<FragmentModule>,
    pub(crate) layout: Arc<wgpu::PipelineLayout>,
}

impl Shader {
    // Create a new shader from the vertex and fragment modules
    pub fn new(
        vertex: &Compiled<VertexModule>,
        fragment: &Compiled<FragmentModule>,
    ) -> Self {
        let modules = &[vertex.reflected(), fragment.reflected()];
        let reflected = super::merge_reflected_module(modules);
        Self {
            vertex: vertex.clone(),
            fragment: fragment.clone(),
            reflected: Arc::new(reflected)
        }
    }

    // Get the vertex module
    pub fn vertex(&self) -> &Compiled<VertexModule> {
        &self.vertex
    }

    // Get the fragment module
    pub fn fragment(&self) -> &Compiled<FragmentModule> {
        &self.fragment
    }
}

// A compute shader used for general computing work
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
}
