use crate::{
    Compiled, Compiler, ComputeModule, FragmentModule, Graphics,
    ShaderCompilationError, ShaderError, VertexModule,
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
    graphics: Graphics,
}

impl Shader {
    // Create a new shader from the vertex and fragment source modules
    pub fn new(
        vertex: VertexModule,
        fragment: FragmentModule,
        compiler: &Compiler,
    ) -> Result<Self, ShaderError> {
        let vertex = compiler.compile(vertex)?;
        let fragment = compiler.compile(fragment)?;
        let names = [vertex.name(), fragment.name()];
        let visibility = [vertex.visibility(), fragment.visibility()];

        todo!()
        /*
        let (reflected, layout) = compiler.create_pipeline_layout(&names, &modules, &visibility)?;

        Ok(Self {
            vertex,
            fragment,
            layout,
            graphics: compiler.graphics.clone(),
        })
        */
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
// This is used for compute work, and nothing else.
// Shaders are clonable since they can be shared between multiple graphics pipelines
#[derive(Clone)]
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
    pub(crate) pipeline: Arc<wgpu::ComputePipeline>,
    graphics: Graphics,
}

impl ComputeShader {
    // Create a new compute shader from the compute module
    pub fn new(module: ComputeModule, compiler: &Compiler) -> Result<Self, ShaderError> {
        let compiled = compiler.compile(module)?;
        let names = [compiled.name()];
        let visibility = [compiled.visibility()];
        todo!()
    }

    // Get the compute module
    pub fn compute(&self) -> &Compiled<ComputeModule> {
        &self.compiled
    }

    // Get the underlying compute pipeline
    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &&self.pipeline
    }
}
