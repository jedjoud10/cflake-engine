use std::{marker::PhantomData, mem::size_of};

use veclib::{Vector, VectorElemCount};

use crate::utils::DataType;

// A stored custom verted data buffer
pub(crate) struct StoredCustomVertexDataBuffer {
    // The vector that stores all the bytes
    pub(crate) inner: Vec<u8>,
    pub(crate) size_per_component: usize,
    pub(crate) _type: DataType,
}

// Some custom vertex data that we can store
pub struct CustomVertexDataBuffer<T, U: Vector<T> + VectorElemCount> {
    pub(crate) inner: Vec<U>,
    pub(crate) _type: DataType,
    _phantom: PhantomData<T>,
}

impl<T, U: Vector<T> + VectorElemCount> CustomVertexDataBuffer<T, U> {
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn new(size: usize, _type: DataType) -> Self {
        Self {
            inner: Vec::with_capacity(size * size_of::<U>()),
            _phantom: PhantomData::default(),
            _type,
        }
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn push(&mut self, vertex_data: U) {
        self.inner.push(vertex_data);
    }
}
