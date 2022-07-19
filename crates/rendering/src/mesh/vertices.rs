use crate::buffer::ArrayBuffer;

use super::attributes::*;

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) colors: &'a AttributeBuffer<Color>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
    pub(super) bitfield: &'a EnabledAttributes,
}

impl VerticesRef<'_> {
    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: VertexAttribute>(&self) -> bool {
        todo!()
    }

    // Get an immutable reference to an attribute buffer
    pub fn data<T: VertexAttribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        todo!()
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        todo!()
    }
}

// Mutable access to the mesh vertices
pub struct VerticesMut<'a> {
    pub(super) vao: u32,
    pub(super) bitfield: &'a mut EnabledAttributes,
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) colors: &'a AttributeBuffer<Color>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
}

impl VerticesMut<'_> {
    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: VertexAttribute>(&self) -> bool {
        todo!()
    }

    // Get an immutable reference to an attribute buffer
    pub fn data<T: VertexAttribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        todo!()
    }

    // Get a mutable reference to an attribute buffer
    pub fn data_mut<T: VertexAttribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        todo!()
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        todo!()
    }
}

impl Drop for VerticesMut<'_> {
    fn drop(&mut self) {
        todo!()
    }
}