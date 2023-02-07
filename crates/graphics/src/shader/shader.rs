use crate::{Compiled, ComputeModule, FragmentModule, VertexModule};
use std::sync::Arc;

// A rendering shader that contains a vertex module and fragment module
// This shader might contain more modules though, like the tesselation modules
#[derive(Clone)]
pub struct Shader {
    vert: Compiled<VertexModule>,
    frag: Compiled<FragmentModule>,
}

impl Shader {
    // Create a new shader from the vertex and fragment modules
    pub fn new(
        vert: &Compiled<VertexModule>,
        frag: &Compiled<FragmentModule>,
    ) -> Self {
        Self {
            vert: vert.clone(),
            frag: frag.clone(),
        }
    }

    // Get the vertex module
    pub fn vertex(&self) -> &Compiled<VertexModule> {
        &self.vert
    }

    // Get the fragment module
    pub fn fragment(&self) -> &Compiled<FragmentModule> {
        &self.frag
    }
}

// A compute shader used for general computing work
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
}
