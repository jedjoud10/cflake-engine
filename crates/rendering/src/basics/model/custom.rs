use core::slice;
use std::{any::TypeId, mem::size_of};

use veclib::{UnsizedVector, Vector, VectorElemCount, SupportedValue};

// Get the bytes of a specific sized generic
fn get_bytes<T: Sized + SupportedValue + 'static, U: Vector<T> + 'static>(data: &U) -> Vec<u8> {
    let slice = unsafe {
        slice::from_raw_parts(data.as_ptr() as *const u8, size_of::<U>())
    };
    // Convert endianness if needed
    let mut vec = slice.to_vec(); 
    vec.reverse();
    vec
}

// Some custom vertex data that we can store
#[derive(Default)]
pub struct CustomVertexDataBuffer {
    pub(crate) inner: Option<Vec<u8>>,
    pub(crate) _type: Option<TypeId>,
    pub(crate) size_pre_component: usize,
}

impl CustomVertexDataBuffer {
    // Set this custom vertex data buffer's type
    fn set_type<T: Sized + 'static, U: Vector<T> + VectorElemCount>(&mut self) {
        // Set the size in bytes of our type
        self._type = Some(TypeId::of::<T>());
        self.inner = Some(Vec::new());
        self.size_pre_component = U::ELEM_COUNT;
    }
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn allocate<T: Sized + 'static, U: Vector<T> + VectorElemCount>(&mut self, size: usize) {
        // Set the type if it wasn't set already
        if self._type.is_none() { self.set_type::<T, U>() }
        *self.inner.as_mut().unwrap() = Vec::with_capacity(size * size_of::<U>());
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn add<T: Sized + SupportedValue + 'static, U: Vector<T> + VectorElemCount + 'static>(&mut self, vertex_data: U) {
        // Set the type if it wasn't set already
        if self._type.is_none() { self.set_type::<T, U>() }
        // But if it was already set, we must check
        else if self._type.unwrap() != TypeId::of::<T>() { panic!(); }

        self.add_unchecked(vertex_data);
    }
    // Add a single custom vertex data, but without checking for type equality
    pub fn add_unchecked<T: Sized + SupportedValue + 'static, U: Vector<T> + 'static>(&mut self, vertex_data: U) {
        // Get the bytes first
        let bytes = get_bytes(&vertex_data);
        dbg!(bytes.len());
        self.inner.as_mut().unwrap().extend(bytes);
    }
    // Check if we have some custom data stored
    pub fn valid(&self) -> bool {
        self.inner.is_some()
    }
}