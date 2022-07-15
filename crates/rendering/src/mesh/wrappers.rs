use crate::buffer::{ElementBuffer, BufferFormatAny, ArrayBuffer};
use super::{EnabledAttributes, AttributeFormatAny, VertexAttribute, AttributeBuffer, TexCoord, Color, Tangent, Normal, Position};

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
    // Get a reference to a vertex attribute buffer
    pub fn attribute_buffer<T: VertexAttribute>(&self) -> Option<ArrayBuffer<T::Out>> {
        None
    }

    // Check if a vertex attribute is active
    pub fn is_attribute_enabled<T: VertexAttribute>(&self) -> bool {
        todo!()
    }
    
    // Get the BufferAnyRef and AttributeFormatAny wrappers for an attribute buffer
    pub fn attribute_any<T: VertexAttribute>(&self) -> Option<(BufferFormatAny, AttributeFormatAny)> {
        todo!()
    }
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