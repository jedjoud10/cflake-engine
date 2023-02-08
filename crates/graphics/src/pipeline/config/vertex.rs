use std::marker::PhantomData;

use wgpu::{VertexFormat, VertexStepMode, ShaderLocation};
use crate::{VertexInfo, Vertex};

// This vertex config describes how vertices or more specifically, "vertex buffers" should be read
// This maps the vertex buffer bindings directly to be usable by a render pass 
// TODO: Implement custom shader locations
// PerVertex<XYZ<f32>>;
// PerInstance<XYZW<f32>>;
// PerVertex<Vert>;
pub struct VertexConfig {
    pub attributes: Vec<VertexAttributeInfo>,
}

// Contains the vertex attribute for a single vertex buffer
pub struct VertexAttributeInfo {
    info: VertexInfo,
    step_mode: VertexStepMode,
}

// Defines a Vertex / a struct that contains multiple Vertices (must be #repr[C])
pub trait VertexLayout {
    fn vertex_infos() -> Vec<
}

// Defines a Vertex buffer/layout as being an input that
// should be updated for every vertex drawn in the mesh
pub struct PerVertex<V: VertexLayout>(PhantomData<V>);

impl<V: Vertex> PerVertex<V> {
    pub fn info() -> VertexAttributeInfo {
        VertexAttributeInfo {
            info: V::info(),
            step_mode: VertexStepMode::Vertex
        }
    }
}

// Defines a Vertex buffer/layout as being an input
// that should be updated for every instance drawn
pub struct PerInstance<V: VertexLayout>(PhantomData<V>);