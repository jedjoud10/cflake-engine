use crate::{
    Compiled, Compiler, ComputeModule, FragmentModule, Graphics, ReflectedShader,
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
    pub(crate) reflected: Arc<ReflectedShader>,
    graphics: Graphics,
}

impl Drop for Shader {
    fn drop(&mut self) {
        if Arc::strong_count(&self.reflected) == 2 {
            //self.graphics.drop_cached_pipeline_layout(&self.reflected);
        }
    }
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
        let modules = [vertex.reflected(), fragment.reflected()];
        let visibility = [vertex.visibility(), fragment.visibility()];
        let (reflected, layout) = compiler.create_pipeline_layout(&names, &modules, &visibility)?;

        Ok(Self {
            vertex,
            fragment,
            layout,
            reflected,
            graphics: compiler.graphics.clone(),
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
    pub(crate) pipeline: Arc<wgpu::ComputePipeline>,
    graphics: Graphics,
}

impl Drop for ComputeShader {
    fn drop(&mut self) {
        if Arc::strong_count(&self.reflected) == 2 {
            //self.graphics.drop_cached_pipeline_layout(&self.reflected);
        }
    }
}

impl ComputeShader {
    // Create a new compute shader from the compute module
    pub fn new(module: ComputeModule, compiler: &Compiler) -> Result<Self, ShaderError> {
        let compiled = compiler.compile(module)?;
        let names = [compiled.name()];
        let modules = [compiled.reflected()];
        let visibility = [compiled.visibility()];
        let (reflected, layout) = compiler.create_pipeline_layout(&names, &modules, &visibility)?;

        let pipeline =
            compiler
                .graphics
                .device()
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(&format!("compute-pipeline-{:?}", compiled.name())),
                    layout: Some(&layout),
                    module: compiled.module(),
                    entry_point: "main",
                });

        Ok(Self {
            pipeline: Arc::new(pipeline),
            compiled,
            layout,
            reflected,
            graphics: compiler.graphics.clone(),
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

    // Get the underlying compute pipeline
    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &&self.pipeline
    }
}
