use std::marker::PhantomData;

use wgpu::{VertexFormat, VertexStepMode, ShaderLocation};
use crate::{VertexInfo, Vertex};

// This vertex config describes how vertices or more specifically, "vertex buffers" should be read
// This maps the vertex buffer bindings directly to be usable by a render pass 
pub struct VertexConfig<'a, 'b> {
    pub inputs: &'a [&'b dyn VertexInput],
}

// Vertex input defines the vertex layout for a single buffer
// TODO: Implement vertex interlacing
pub trait VertexInput {
    // Create a new vertex input (layout) using the appropriate shader location
    fn new(location: ShaderLocation) -> Self where Self: Sized;

    // Get the shader location of the vertex input
    fn location(&self) -> ShaderLocation;

    // Compile time Vertex and step mode
    fn info(&self) -> VertexInfo;
    fn step_mode(&self) -> VertexStepMode;
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

    fn info(&self) -> VertexInfo {
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

    fn info(&self) -> VertexInfo {
        V::info()
    }

    fn step_mode(&self) -> VertexStepMode {
        VertexStepMode::Instance
    }
}