use graphics::VertexBuffer;
use math::AABB;
use crate::MeshAabbComputeError;
use super::attributes::*;

// Immutable access to the mesh vertices
#[cfg(not(feature = "two-dim"))]
pub struct VerticesRef<'a> {
    pub(super) enabled: EnabledMeshAttributes,
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) tex_coords: &'a AttributeBuffer<TexCoord>,
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
        T::from_ref_as_ref(self)
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
    pub(super) tex_coords: &'a mut AttributeBuffer<TexCoord>,
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
        T::from_mut_as_ref(self)
    }

    // Get a mutable reference to an attribute buffer
    pub fn attribute_mut<T: MeshAttribute>(&mut self) -> Option<&mut VertexBuffer<T::Storage>> {
        T::from_mut_as_mut(self)
    }

    // Insert a new vertex buffer to the vertices
    pub fn insert<T: MeshAttribute>(&mut self, buffer: VertexBuffer<T::Storage>) {
        T::insert(self, buffer);
    }

    // Remove an old vertex buffer from the vertices
    pub fn remove<T: MeshAttribute>(&mut self) -> Option<VertexBuffer<T::Storage>> {
        T::remove(self)
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        todo!()
    }

    // Try to compute the AABB of the mesh using updated position vertices
    pub fn compute_aabb(&mut self) -> Result<AABB, MeshAabbComputeError> {
        todo!()
    }
}