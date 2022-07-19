use crate::buffer::ArrayBuffer;

use super::attributes::*;

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) bitfield: &'a EnabledAttributes,
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) colors: &'a AttributeBuffer<Color>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
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
    pub(super) maybe_reassigned: EnabledAttributes,
    pub(super) positions: &'a mut AttributeBuffer<Position>,
    pub(super) normals: &'a mut AttributeBuffer<Normal>,
    pub(super) tangents: &'a mut AttributeBuffer<Tangent>,
    pub(super) colors: &'a mut AttributeBuffer<Color>,
    pub(super) uvs: &'a mut AttributeBuffer<TexCoord>,
}

impl VerticesMut<'_> {
    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: VertexAttribute>(&self) -> bool {
        self.bitfield.contains(T::ENABLED)
    }

    // Get an immutable reference to an attribute buffer
    pub fn data<T: VertexAttribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        unsafe {
            self.is_enabled::<T>()
                .then(|| T::from_vertices_mut_as_ref(self).assume_init_ref())
        }
    }

    // Get a mutable reference to an attribute buffer
    pub fn data_mut<T: VertexAttribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        unsafe {
            self.is_enabled::<T>()
                .then(|| T::from_vertices_mut_as_mut(self).assume_init_mut())
        }
    }

    // Set a new attribute buffer
    pub fn set_data<T: VertexAttribute>(&mut self, buffer: Option<ArrayBuffer<T::Out>>) {
        if let Some(buffer) = buffer {
            unsafe {
                T::insert(self, buffer);
                gl::EnableVertexArrayAttrib(self.vao, T::index());
                gl::VertexArrayAttribFormat(
                    self.vao,
                    T::index(),
                    T::COUNT_PER_VERTEX as i32,
                    T::GL_TYPE,
                    T::NORMALIZED.into(),
                    0,
                );
            }
        } else {
            T::remove(self);
            unsafe { gl::DisableVertexArrayAttrib(self.vao, T::index()) }
        }
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
