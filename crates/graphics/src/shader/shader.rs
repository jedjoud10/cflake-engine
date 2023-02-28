use crate::{
    Compiled, ComputeModule, FragmentModule, Graphics,
    ReflectedModule, ReflectedShader, VertexModule,
};
use std::sync::Arc;

// A rendering shader that contains a vertex module and fragment module
// This shader might contain more modules though, like the tesselation modules
#[derive(Clone)]
pub struct Shader {
    vertex: Compiled<VertexModule>,
    fragment: Compiled<FragmentModule>,
    pub(crate) layout: Arc<wgpu::PipelineLayout>,
    pub(crate) reflected: Arc<ReflectedShader>,
}

impl Shader {
    // Create a new shader from the vertex and fragment modules
    pub fn new(
        graphics: &Graphics,
        vertex: &Compiled<VertexModule>,
        fragment: &Compiled<FragmentModule>,
    ) -> Self {
        let (shader, layout) = super::merge_and_make_layout(vertex, fragment, graphics);


        Self {
            vertex: vertex.clone(),
            fragment: fragment.clone(),
            layout,
            reflected: Arc::new(shader),
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

    // Get the underlying reflected shader
    pub fn reflected(&self) -> &ReflectedShader {
        &self.reflected
    }
}

// A compute shader used for general computing work
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
}
