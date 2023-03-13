use std::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
};

use super::attributes::*;
use crate::MeshAabbComputeError;
use graphics::{Buffer, BufferInfo, CommandEncoder, VertexBuffer};
use math::AABB;

// Immutable access to the mesh vertices
pub struct VerticesRef<'a> {
    pub(super) enabled: EnabledMeshAttributes,
    pub(super) positions: &'a MaybeUninit<AttributeBuffer<Position>>,
    pub(super) normals: &'a MaybeUninit<AttributeBuffer<Normal>>,
    pub(super) tangents: &'a MaybeUninit<AttributeBuffer<Tangent>>,
    pub(super) tex_coords: &'a MaybeUninit<AttributeBuffer<TexCoord>>,
    pub(super) len: Option<usize>,
}

impl<'a> VerticesRef<'a> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled(&self) -> EnabledMeshAttributes {
        self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(
        &self,
    ) -> Option<&'a VertexBuffer<T::V>> {
        T::from_ref_as_ref(self)
    }

    // Get all the available attribute buffers as untyped buffers types
    pub fn untyped_buffers(
        &self,
    ) -> [Option<BufferInfo>; MAX_MESH_VERTEX_ATTRIBUTES] {
        [
            self.attribute::<Position>().map(Buffer::as_untyped),
            self.attribute::<Normal>().map(Buffer::as_untyped),
            self.attribute::<Tangent>().map(Buffer::as_untyped),
            //self.attribute::<Color>().map(|b| Buffer::untyped(b)),
            self.attribute::<TexCoord>().map(Buffer::as_untyped),
        ]
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        self.len
    }
}

// Mutable access to the mesh vertices
pub struct VerticesMut<'a> {
    pub(super) enabled: &'a mut EnabledMeshAttributes,
    pub(super) positions:
        &'a mut MaybeUninit<AttributeBuffer<Position>>,
    pub(super) normals: &'a mut MaybeUninit<AttributeBuffer<Normal>>,
    pub(super) tangents:
        &'a mut MaybeUninit<AttributeBuffer<Tangent>>,
    pub(super) tex_coords:
        &'a mut MaybeUninit<AttributeBuffer<TexCoord>>,
    pub(super) len: RefCell<&'a mut Option<usize>>,
    pub(super) dirty: Cell<bool>,
}

impl<'a> VerticesMut<'a> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled(&self) -> EnabledMeshAttributes {
        *self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(
        &self,
    ) -> Option<&AttributeBuffer<T>> {
        T::from_mut_as_ref(self)
    }

    // Get a mutable reference to an attribute buffer
    pub fn attribute_mut<T: MeshAttribute>(
        &mut self,
    ) -> Option<&mut AttributeBuffer<T>> {
        self.dirty.set(true);
        T::from_mut_as_mut(self)
    }

    // Insert a new vertex buffer to the vertices
    pub fn insert<T: MeshAttribute>(
        &mut self,
        buffer: AttributeBuffer<T>,
    ) {
        self.dirty.set(true);
        T::insert(self, buffer);
    }

    // Remove an old vertex buffer from the vertices
    pub fn remove<T: MeshAttribute>(
        &mut self,
    ) -> Option<AttributeBuffer<T>> {
        T::remove(self)
    }

    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        if self.dirty.get() {
            // Fetch the length of each of the attribute (even if they don't actually exist)
            let positions =
                self.attribute::<Position>().map(|x| x.len());
            let normals = self.attribute::<Normal>().map(|x| x.len());
            let tangents =
                self.attribute::<Tangent>().map(|x| x.len());
            let tex_coords =
                self.attribute::<TexCoord>().map(|x| x.len());

            // Convert the options into a fixed sized array and iterate over it
            let array = [positions, normals, tangents, tex_coords];
            let length = array
                .into_iter()
                .reduce(|accum, actual| {
                    if accum.is_some() && accum == actual {
                        accum
                    } else {
                        None
                    }
                })
                .unwrap();

            // Update and remove the "dirty" state
            **self.len.borrow_mut() = length;
            self.dirty.set(true);
        }

        **self.len.borrow()
    }

    // Try to compute the AABB of the mesh using updated position vertices
    pub fn compute_aabb(
        &mut self,
        _encoder: &mut CommandEncoder,
    ) -> Result<AABB, MeshAabbComputeError> {
        let attribute = self.attribute::<Position>().ok_or(
            MeshAabbComputeError::MissingPositionAttributeBuffer,
        )?;
        let view = attribute
            .as_view(..)
            .map_err(MeshAabbComputeError::NotHostMapped)?;
        let slice = view.as_slice();
        let aabb = super::aabb_from_points(slice).ok_or(
            MeshAabbComputeError::EmptyPositionAttributeBuffer,
        )?;
        Ok(aabb)
    }
}
