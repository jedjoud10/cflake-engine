use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    mem::MaybeUninit,
};

use super::attributes::*;
use crate::{AttributeError, MeshAabbComputeError, RenderPath, Direct, Indirect};
use graphics::{Buffer, BufferInfo, VertexBuffer, DrawIndexedIndirectBuffer};
use math::Aabb;
use utils::Handle;

// Immutable access to the mesh vertices
pub struct VerticesRef<'a, R: RenderPath> {
    pub(super) enabled: MeshAttributes,
    pub(super) positions: &'a Option<R::AttributeBuffer<Position>>,
    pub(super) normals: &'a Option<R::AttributeBuffer<Normal>>,
    pub(super) tangents: &'a Option<R::AttributeBuffer<Tangent>>,
    pub(super) tex_coords: &'a Option<R::AttributeBuffer<TexCoord>>,
    pub(super) count: &'a R::Count,
    pub(super) aabb: Option<math::Aabb<f32>>,
}

impl<'a, R: RenderPath> VerticesRef<'a, R> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled(&self) -> MeshAttributes {
        self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(
        &self,
    ) -> Result<&'a R::AttributeBuffer<T>, AttributeError> {
        T::from_ref_as_ref(self)
    }

    // Get the axis-aligned bounding box for this mesh
    // Returns None if the AABB wasn't computed yet or if computation failed
    pub fn aabb(&mut self) -> Option<math::Aabb<f32>> {
        self.aabb
    }
}

impl<'a> VerticesRef<'a, Direct> {
    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        *self.count
    }
}

// Mutable access to the mesh vertices
pub struct VerticesMut<'a, R: RenderPath> {
    // Attributes
    pub(super) enabled: &'a mut MeshAttributes,
    pub(super) positions:
        RefCell<&'a mut Option<R::AttributeBuffer<Position>>>,
    pub(super) normals:
        RefCell<&'a mut Option<R::AttributeBuffer<Normal>>>,
    pub(super) tangents:
        RefCell<&'a mut Option<R::AttributeBuffer<Tangent>>>,
    pub(super) tex_coords:
        RefCell<&'a mut Option<R::AttributeBuffer<TexCoord>>>,

    // Cached parameters
    pub(super) count: RefCell<&'a mut R::Count>,
    pub(super) aabb: RefCell<&'a mut Option<math::Aabb<f32>>>,

    // Parameters to keep track of cached data
    pub(super) length_dirty: Cell<bool>,
    pub(super) aabb_dirty: Cell<bool>,
}

impl<'a, P: RenderPath> VerticesMut<'a, P> {
    // Get the enabled mesh attributes bitflags
    pub fn enabled(&self) -> MeshAttributes {
        *self.enabled
    }

    // Check if an attribute buffer is enabled
    pub fn is_enabled<T: MeshAttribute>(&self) -> bool {
        self.enabled.contains(T::ATTRIBUTE)
    }

    // Get an immutable reference to an attribute buffer
    pub fn attribute<T: MeshAttribute>(
        &self,
    ) -> Result<Ref<P::AttributeBuffer<T>>, AttributeError> {
        T::from_mut_as_ref(self)
    }

    // Get a mutable reference to an attribute buffer
    pub fn attribute_mut<T: MeshAttribute>(
        &self,
    ) -> Result<RefMut<P::AttributeBuffer<T>>, AttributeError> {
        self.set_as_dirty::<T>();
        T::from_mut_as_mut(self)
    }

    // Insert a new vertex buffer to the vertices
    pub fn insert<T: MeshAttribute>(
        &mut self,
        buffer: P::AttributeBuffer<T>,
    ) {
        self.set_as_dirty::<T>();
        T::insert(self, buffer);
    }

    // Remove an old vertex buffer from the vertices
    pub fn remove<T: MeshAttribute>(
        &mut self,
    ) -> Result<P::AttributeBuffer<T>, AttributeError> {
        self.set_as_dirty::<T>();
        T::remove(self)
    }

    // Called whenever we access an attribute mutably
    // Only used internally to set the dirty states
    fn set_as_dirty<T: MeshAttribute>(&self) {
        self.length_dirty.set(true);

        if T::ATTRIBUTE.contains(MeshAttributes::POSITIONS) {
            self.aabb_dirty.set(true);
        }
    }
}

impl<'a> VerticesMut<'a, Direct> {
    // Get the number of vertices that we have (will return None if we have buffers of mismatching lengths)
    pub fn len(&self) -> Option<usize> {
        if self.length_dirty.take() {
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
                .map(|x| x.ok())
                .reduce(|accum, actual| {
                    if accum.is_some() && accum == actual {
                        accum
                    } else {
                        None
                    }
                })
                .unwrap();

            // Update length
            **self.count.borrow_mut() = length;
        }

        **self.count.borrow()
    }

    // Calculate an Axis-Aligned Bounding Box, and returns an error if not possible
    pub fn aabb(
        &self,
    ) -> Result<math::Aabb<f32>, MeshAabbComputeError> {
        if self.aabb_dirty.take() {
            // Fetch the position attribute buffer
            let attribute =
                self.attribute::<Position>().map_err(|x| {
                    MeshAabbComputeError::AttributeBuffer(x)
                })?;

            // Create a view into the buffer (if possible)
            let view = attribute
                .as_view(..)
                .map_err(MeshAabbComputeError::NotHostMapped)?;

            // Create a visible rust slice of the buffer view
            let slice = view.as_slice();

            // Generate the AABB from the buffer view
            **self.aabb.borrow_mut() = Some(super::aabb_from_points(slice).ok_or(
                MeshAabbComputeError::EmptyPositionAttributeBuffer,
            )?);
        }

        Ok(self.aabb.borrow().unwrap())
    }
}

impl<'a> VerticesMut<'a, Indirect> {
    // Get the indexed indirect buffer handle mutably
    pub fn indirect_mut(&mut self) -> &mut Handle<DrawIndexedIndirectBuffer> {
        self.count.get_mut()
    }

    // Get the indexed indirect buffer handle immutably
    pub fn indirect(&self) -> Handle<DrawIndexedIndirectBuffer> {
        self.count.borrow().clone()
    }
}