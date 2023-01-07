use graphics::VertexBuffer;
use math::AABB;
use crate::MeshAabbComputeError;

use super::{attributes::*, MeshUtils};

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) enabled: EnabledMeshAttributes,
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
    pub(super) len: Option<usize>,
}

impl VerticesRef<'_> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled_attributes(&self) -> EnabledMeshAttributes {
        self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(&self) -> Option<&VertexBuffer<T::Storage>> {
        todo!()
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        self.len
    }
}

// Mutable access to the mesh vertices
pub struct VerticesMut<'a> {
    pub(super) enabled: &'a mut EnabledMeshAttributes,
    pub(super) positions: &'a mut AttributeBuffer<Position>,
    pub(super) normals: &'a mut AttributeBuffer<Normal>,
    pub(super) tangents: &'a mut AttributeBuffer<Tangent>,
    pub(super) uvs: &'a mut AttributeBuffer<TexCoord>,
    pub(super) len: &'a mut Option<usize>,
}

impl VerticesMut<'_> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled_attributes(&self) -> EnabledMeshAttributes {
        *self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(&self) -> Option<&VertexBuffer<T::Storage>> {
        todo!()
    }

    // Get a mutable reference to an attribute buffer
    pub fn attribute_mut<T: MeshAttribute>(&mut self) -> Option<&mut VertexBuffer<T::Storage>> {
        todo!()
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        todo!()
    }

    // Set a new attribute buffer (this ignores that the buffer is a different length)
    pub fn set_attribute<T: MeshAttribute>(&mut self, buffer: Option<VertexBuffer<T::Storage>>) {
        todo!()
    }

    // Try to compute the AABB of the mesh using updated position vertices
    pub fn compute_aabb(&mut self) -> Result<AABB, MeshAabbComputeError> {
        todo!()
    }
}