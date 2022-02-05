use std::mem::ManuallyDrop;

use crate::utils::DataType;

// Stored version of the custom data, just stores the elements are raw bytes
pub struct StoredCustomVertexDataBuffer {
    pub(crate) inner: Vec<u8>,
    pub(crate) components_per_vertex: usize,
    pub(crate) _type: DataType,
}

impl StoredCustomVertexDataBuffer {
    // Create a new stored buffer from a custom one
    pub fn new<T>(custom: CustomVertexDataBuffer<T>, _type: DataType) -> Self {
        let mut vec = ManuallyDrop::new(custom.inner);
        let len = std::mem::size_of::<T>() * vec.len();
        // Create a new vector
        let inner = unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut u8, len, len) };
        Self {
            inner,
            components_per_vertex: custom.components_per_vertex,
            _type,
        }
    }
}

// Some custom vertex data that we can store
pub struct CustomVertexDataBuffer<T> {
    pub(crate) inner: Vec<T>,
    pub(crate) components_per_vertex: usize,
}

impl<T> CustomVertexDataBuffer<T> {
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn with_capacity(capacity: usize, elem_count: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            components_per_vertex: elem_count,
        }
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn push(&mut self, vertex_data: T) {
        self.inner.push(vertex_data);
    }
}
