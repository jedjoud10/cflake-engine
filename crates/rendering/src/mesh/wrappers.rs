use super::{
    AttributeBuffer, AttributeFormatAny, Color, EnabledAttributes, Normal, Position, Tangent,
    TexCoord, VertexAttribute,
};
use crate::buffer::{ArrayBuffer, BufferFormatAny, ElementBuffer};

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) colors: &'a AttributeBuffer<Color>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
    pub(super) bitfield: &'a EnabledAttributes,
}

impl<'a> VerticesRef<'a> {
}

// Mutable access to the mesh vertices
pub struct VerticesMut<'a> {
    vao: u32,
    bitfield: &'a mut EnabledAttributes,
}

// Immutable access to the mesh indices
pub struct IndicesRef<'a> {
    buffer: &'a ElementBuffer<u32>,
}

// Mutable access to the mesh indices
pub struct IndicesMut<'a> {
    vao: u32,
    buffer: &'a ElementBuffer<u32>,
}
