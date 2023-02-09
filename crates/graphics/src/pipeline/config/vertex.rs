use std::marker::PhantomData;

use wgpu::{VertexFormat, VertexStepMode, ShaderLocation};
use crate::{VertexInfo, Vertex};

// This vertex config describes how vertices or more specifically, "vertex buffers" should be read
// This maps the vertex buffer bindings directly to be usable by a render pass 
pub struct VertexConfig<'a, 'b> {
    pub inputs: &'a [&'b dyn VertexInput],
}

// VertexInputInfo combines all the required info for a vertex input in one struct
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexInputInfo {
    location: ShaderLocation,
    info: VertexInfo,
    step_mode: VertexStepMode,
}

impl VertexInputInfo {
    // Get the shader location of the VertexInput
    pub fn location(&self) -> ShaderLocation {
        self.location
    }

    // Get the VertexInfo of the VertexInput
    pub fn vertex_info(&self) -> VertexInfo {
        self.info
    }

    // Get the step mode of the VertexInput
    pub fn step_mode(&self) -> VertexStepMode {
        self.step_mode
    }
}

// Vertex input defines the vertex layout for a single buffer
// TODO: Implement vertex interlacing
pub trait VertexInput {
    // Create a new vertex input (layout) using the appropriate shader location
    fn new(location: ShaderLocation) -> Self where Self: Sized;

    // Get the shader location of the vertex input
    fn location(&self) -> ShaderLocation;

    // Compile time Vertex and step mode
    fn vertex_info(&self) -> VertexInfo;
    fn step_mode(&self) -> VertexStepMode;

    // Get the combined info
    fn info(&self) -> VertexInputInfo {
        VertexInputInfo {
            location: self.location(),
            info: self.vertex_info(),
            step_mode: self.step_mode(),
        }
    }
}

// Defines a Vertex buffer/layout as being an input that
// should be updated for every vertex drawn in the mesh
pub struct PerVertex<V: Vertex>(PhantomData<V>, ShaderLocation);
impl<V: Vertex> VertexInput for PerVertex<V> {
    fn new(location: ShaderLocation) -> Self {
        Self(PhantomData, location)
    }

    fn location(&self) -> ShaderLocation {
        self.1
    }

    fn vertex_info(&self) -> VertexInfo {
        V::info()
    }

    fn step_mode(&self) -> VertexStepMode {
        VertexStepMode::Vertex
    }
}

// Defines a Vertex buffer/layout as being an input
// that should be updated for every instance drawn
pub struct PerInstance<V: Vertex>(PhantomData<V>, ShaderLocation);
impl<V: Vertex> VertexInput for PerInstance<V> {
    fn new(location: ShaderLocation) -> Self {
        Self(PhantomData, location)
    }

    fn location(&self) -> ShaderLocation {
        self.1
    }

    fn vertex_info(&self) -> VertexInfo {
        V::info()
    }

    fn step_mode(&self) -> VertexStepMode {
        VertexStepMode::Instance
    }
}