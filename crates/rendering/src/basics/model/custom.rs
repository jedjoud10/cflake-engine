use core::slice;
use std::{any::TypeId, mem::size_of};

// Get the bytes of a specific sized generic
fn get_bytes<T: Sized + 'static>(data: &T) -> &[u8] {
    unsafe {
        slice::from_raw_parts(data as *const T as *const u8, size_of::<T>())
    }
}

// Some custom vertex data that we can store
#[derive(Default)]
pub struct CustomVertexDataBuffer {
    pub(crate) inner: Option<Vec<u8>>,
    pub(crate) _type: Option<TypeId>,
}

impl CustomVertexDataBuffer {
    // Set this custom vertex data buffer's type
    fn set_type<T: Sized + 'static>(&mut self) {
        // Set the size in bytes of our type
        self._type = Some(TypeId::of::<T>());
        self.inner = Some(Vec::new());
    }
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn allocate<T: Sized + 'static>(&mut self, size: usize) {
        // Set the type if it wasn't set already
        if self._type.is_none() { self.set_type::<T>() }
        *self.inner.as_mut().unwrap() = Vec::with_capacity(size * size_of::<T>());
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn add<T: Sized + 'static>(&mut self, vertex_data: T) {
        // Set the type if it wasn't set already
        if self._type.is_none() { self.set_type::<T>() }
        // But if it was already set, we must check
        else if self._type.unwrap() != TypeId::of::<T>() { panic!(); }

        self.add_unchecked(vertex_data);
    }
    // Add a single custom vertex data, but without checking for type equality
    pub fn add_unchecked<T: Sized + 'static>(&mut self, vertex_data: T) {
        // Get the bytes first
        let bytes = get_bytes(&vertex_data);
        self.inner.as_mut().unwrap().extend_from_slice(bytes);
    }
}