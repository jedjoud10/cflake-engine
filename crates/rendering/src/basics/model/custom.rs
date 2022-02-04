use std::{marker::PhantomData, mem::size_of};

use veclib::{Vector, VectorElemCount};

use crate::utils::DataType;

// A stored custom verted data buffer
pub(crate) struct StoredCustomVertexDataBuffer {
    // The vector that stores all the bytes
    pub(crate) inner: Vec<u8>,
    pub(crate) components_per_vertex: usize,
}

// Some custom vertex data that we can store
pub struct CustomVertexDataBuffer<T> {
    pub(crate) inner: Vec<T>,
    pub(crate) components_per_vertex: usize,
}

impl<T> CustomVertexDataBuffer<T> {
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn with_capacity<U>(capacity: usize) -> Self where T: Vector<U> + VectorElemCount {
        Self {
            inner: Vec::with_capacity(capacity),
            components_per_vertex: T::ELEM_COUNT,
        }
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn push<U>(&mut self, vertex_data: T)  {
        self.inner.push(vertex_data);
    }
}
