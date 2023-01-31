use crate::{
    Compiled, CompiledDescription, ComputeModule, FragmentModule,
    VertexModule,
};
use std::sync::Arc;

// A rendering shader that contains a vertex module and fragment module
// This shader might contain more modules though, like the tesselation modules
#[derive(Clone)]
pub struct Shader {
    vert: Arc<Compiled<VertexModule>>,
    frag: Arc<Compiled<FragmentModule>>,
}

impl Shader {
    // Create a new shader from the vertex and fragment modules
    // FIXME: Use shader source sharing
    pub fn new(
        vert: Compiled<VertexModule>,
        frag: Compiled<FragmentModule>,
    ) -> Self {
        Self {
            vert: Arc::new(vert),
            frag: Arc::new(frag),
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

    // Get the untyped module descriptions
    pub fn descriptions(&self) -> [CompiledDescription; 2] {
        [self.vertex().description(), self.fragment().description()]
    }
}

// A compute shader used for general computing work
pub struct ComputeShader {
    compiled: Arc<Compiled<ComputeModule>>,
}
