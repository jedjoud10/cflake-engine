use math::AABB;

use super::{attributes::*, MeshUtils};
use crate::{
    buffer::{ArrayBuffer, Buffer, UntypedBufferFormat},
    context::ToGlName,
};

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) bitfield: EnabledAttributes,
    pub(super) positions: &'a AttributeBuffer<Position>,
    pub(super) normals: &'a AttributeBuffer<Normal>,
    pub(super) tangents: &'a AttributeBuffer<Tangent>,
    pub(super) colors: &'a AttributeBuffer<Color>,
    pub(super) uvs: &'a AttributeBuffer<TexCoord>,
}

impl VerticesRef<'_> {
    // Get the attribute bitfield layout
    pub fn layout(&self) -> EnabledAttributes {
        self.bitfield
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: VertexAttribute>(&self) -> bool {
        self.bitfield.contains(T::ENABLED)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: VertexAttribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        unsafe {
            self.is_enabled::<T>()
                .then(|| T::from_vertices_ref_as_ref(self).assume_init_ref())
        }
    }

    // Get all the available attribute buffers as any wrapper types
    pub fn as_any(
        &self,
    ) -> [Option<(UntypedBufferFormat, AttributeFormatAny)>; MAX_MESH_VERTEX_ATTRIBUTES] {
        [
            self.attribute::<Position>()
                .map(|b| (Buffer::untyped_format(b), Position::format_any())),
            self.attribute::<Normal>()
                .map(|b| (Buffer::untyped_format(b), Normal::format_any())),
            self.attribute::<Tangent>()
                .map(|b| (Buffer::untyped_format(b), Tangent::format_any())),
            self.attribute::<Color>()
                .map(|b| (Buffer::untyped_format(b), Color::format_any())),
            self.attribute::<TexCoord>()
                .map(|b| (Buffer::untyped_format(b), TexCoord::format_any())),
        ]
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    // TODO: Fix code duplication
    pub fn len(&self) -> Option<usize> {
        let slice = self.as_any();
        let maybe_min = slice.iter().filter_map(|f| f.map(|(b, _)| b.len())).min();
        let maybe_max = slice.iter().filter_map(|f| f.map(|(b, _)| b.len())).max();
        let min = maybe_min.unwrap_or_default();
        let max = maybe_max.unwrap_or_default();
        let valid = min == max;
        valid.then_some(min)
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
    pub(super) aabb: &'a mut Option<AABB>,
}

impl VerticesMut<'_> {
    // Get the attribute bitfield layout
    pub fn layout(&self) -> EnabledAttributes {
        *self.bitfield
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: VertexAttribute>(&self) -> bool {
        self.bitfield.contains(T::ENABLED)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: VertexAttribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        unsafe {
            self.is_enabled::<T>()
                .then(|| T::from_vertices_mut_as_ref(self).assume_init_ref())
        }
    }

    // Get a mutable reference to an attribute buffer
    pub fn attribute_mut<T: VertexAttribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        unsafe {
            self.is_enabled::<T>()
                .then(|| T::from_vertices_mut_as_mut(self).assume_init_mut())
        }
    }

    // Get all the available attribute buffers as any wrapper types
    pub fn as_any(
        &self,
    ) -> [Option<(UntypedBufferFormat, AttributeFormatAny)>; MAX_MESH_VERTEX_ATTRIBUTES] {
        [
            self.attribute::<Position>()
                .map(|b| (Buffer::untyped_format(b), Position::format_any())),
            self.attribute::<Normal>()
                .map(|b| (Buffer::untyped_format(b), Normal::format_any())),
            self.attribute::<Tangent>()
                .map(|b| (Buffer::untyped_format(b), Tangent::format_any())),
            self.attribute::<Color>()
                .map(|b| (Buffer::untyped_format(b), Color::format_any())),
            self.attribute::<TexCoord>()
                .map(|b| (Buffer::untyped_format(b), TexCoord::format_any())),
        ]
    }

    // Set a new attribute buffer (this ignores that the buffer is a different length)
    pub fn set_attribute<T: VertexAttribute>(&mut self, buffer: Option<ArrayBuffer<T::Out>>) {
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
        let slice = self.as_any();
        let maybe_min = slice.iter().filter_map(|f| f.map(|(b, _)| b.len())).min();
        let maybe_max = slice.iter().filter_map(|f| f.map(|(b, _)| b.len())).max();
        let min = maybe_min.unwrap_or_default();
        let max = maybe_max.unwrap_or_default();
        let valid = min == max;
        valid.then_some(min)
    }

    // Update the AABB of the mesh using updated position vertices
    pub fn compute_aabb(&mut self) -> Option<()> {
        let positions = self.attribute::<Position>()?;
        let view = positions.as_view()?;
        let slice = view.as_slice();
        let temp = MeshUtils::aabb_from_points(slice);
        drop(view);
        *self.aabb = temp;

        Some(())
    }

    // Re-bind the vertex buffers to the VAO, assuming that they are valid
    // This is done automatically when self is dropped
    pub fn rebind(&mut self, force: bool) -> bool {
        if self.len().is_none() {
            return false;
        }

        for (i, (buffer, attrib)) in self.as_any().into_iter().flatten().enumerate() {
            if self.maybe_reassigned.contains(attrib.tag()) || force {
                unsafe {
                    // TODO: Just use a state machine internally saved in the mesh bozo
                    gl::VertexArrayAttribBinding(self.vao, attrib.attribute_index(), i as u32);
                    gl::VertexArrayVertexBuffer(
                        self.vao,
                        i as u32,
                        buffer.name(),
                        0,
                        buffer.stride() as i32,
                    );
                }
            }
        }

        true
    }
}

impl Drop for VerticesMut<'_> {
    fn drop(&mut self) {
        self.compute_aabb();
        assert!(self.rebind(false));
    }
}
