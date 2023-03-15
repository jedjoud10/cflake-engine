use crate::{
    Compiled, Compiler, ComputeModule, FragmentModule, Graphics,
    ReflectedShader, ShaderCompilationError, ShaderError,
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
    ) -> Result<Self, ShaderError> {
        let vertex = compiler.compile(vertex, graphics)?;
        let fragment = compiler.compile(fragment, graphics)?;
        let names = [vertex.name(), fragment.name()];
        let modules = [vertex.naga(), fragment.naga()];
        let visibility = [vertex.visibility(), fragment.visibility()];
        let (reflected, layout) = compiler.create_pipeline_layout(
            graphics,
            &names,
            &modules,
            &visibility,
        )?;

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
// This is used for compute work, and nothing else.
// Shaders are clonable since they can be shared between multiple graphics pipelines
#[derive(Clone)]
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
    pub(crate) layout: Arc<wgpu::PipelineLayout>,
    pub(crate) reflected: Arc<ReflectedShader>,
}

impl ComputeShader {
    // Create a new compute shader from the compute module
    pub fn new(
        graphics: &Graphics,
        module: ComputeModule,
        compiler: Compiler,
    ) -> Result<Self, ShaderError> {
        let compiled = compiler.compile(module, graphics)?;
        let names = [compiled.name()];
        let modules = [compiled.naga()];
        let visibility = [compiled.visibility()];
        let (reflected, layout) = compiler.create_pipeline_layout(
            graphics,
            &names,
            &modules,
            &visibility,
        )?;

        Ok(Self {
            compiled,
            layout,
            reflected,
        })
    }

    // Get the compute module
    pub fn compute(&self) -> &Compiled<ComputeModule> {
        &self.compiled
    }

    // Get the underlying reflected shader
    pub fn reflected(&self) -> &ReflectedShader {
        &self.reflected
    }
}
