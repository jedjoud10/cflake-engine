use crate::{
    Compiled, Compiler, ComputeModule, FragmentModule, Graphics,
    ReflectedShader, ShaderCompilationError,
    VertexModule,
};
use std::sync::Arc;

// A rendering shader that contains a vertex module and fragment module
// This is only used for rendering, and nothing else.
// Shaders are clonable since they can be shared between multiple graphics pipelines
#[derive(Clone)]
pub struct Shader {
    // Compiled vertex modules
    vertex: Compiled<VertexModule>,
    fragment: Compiled<FragmentModule>,

    // WGPU layout and reflected layout of the shader
    pub(crate) layout: Arc<wgpu::PipelineLayout>,
    pub(crate) reflected: Arc<ReflectedShader>,
}

impl Shader {
    // Create a new shader from the vertex and fragment source modules
    pub fn new(
        graphics: &Graphics,
        vertex: VertexModule,
        fragment: FragmentModule,
        compiler: Compiler,
    ) -> Result<Self, ShaderCompilationError> {
        let vertex = compiler.compile(vertex, graphics)?;
        let fragment = compiler.compile(fragment, graphics)?;
        let names = [vertex.name(), fragment.name()];
        let modules = [vertex.naga(), fragment.naga()];
        let visibility = [vertex.visibility(), fragment.visibility()];
        let (reflected, layout) =
            compiler.create_pipeline_layout(graphics, &names, &modules, &visibility);

        Ok(Self {
            vertex,
            fragment,
            layout,
            reflected,
        })
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
    pub(crate) layout: Arc<wgpu::PipelineLayout>,
    pub(crate) reflected: Arc<ReflectedShader>,
}
