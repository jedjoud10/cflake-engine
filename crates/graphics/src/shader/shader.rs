use crate::{
    Compiled, ComputeModule, FragmentModule, Graphics,
    ReflectedModule, VertexModule, ReflectedShader,
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
        // Convert the reflected module to a reflected shader
        let modules = &[vertex.reflected(), fragment.reflected()];
        let shader = super::merge_reflected_modules_to_shader(modules);

        // Convert the reflected shader to a layout
        let layout = super::create_pipeline_layout_from_shader(
            graphics, &shader,
        );

        Self {
            vertex: vertex.clone(),
            fragment: fragment.clone(),
            layout: Arc::new(layout),
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
}

// A compute shader used for general computing work
pub struct ComputeShader {
    compiled: Compiled<ComputeModule>,
}
