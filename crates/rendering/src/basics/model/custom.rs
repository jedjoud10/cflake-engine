use core::slice;
use std::{any::TypeId, mem::size_of};

use veclib::{UnsizedVector, Vector, VectorElemCount, SupportedValue};

use crate::utils::DataType;

// Some custom vertex data that we can store
#[derive(Default)]
pub struct CustomVertexDataBuffer {
    // The vector that stores all the bytes
    pub(crate) inner: Option<(Vec<u8>, TypeId, DataType)>,

    // The count of custom datas per vertex
    pub(crate) size_pre_component: usize,
}

impl CustomVertexDataBuffer {
    // Set this custom vertex data buffer's type
    fn set_type<T: Sized + 'static, U: Vector<T> + VectorElemCount>(&mut self, _type: DataType) {
        // Set the size in bytes of our type
        self.inner = Some((Vec::new(), TypeId::of::<T>(), _type));
        self.size_pre_component = U::ELEM_COUNT;
    }
    // Allocate enough size so we can add multiple Ts without the need to reallocate our inner buffer
    // This also clears the vector
    pub fn allocate<T: Sized + 'static, U: Vector<T> + VectorElemCount>(&mut self, size: usize, _type: DataType) {
        // Always set the type
        self.set_type::<T, U>(_type);
        let (vec, _, _) = self.inner.as_mut().unwrap();
        *vec = Vec::with_capacity(size * size_of::<U>());
    }
    // Add a single custom vertex data, but check if the types match first
    pub fn add<T: Sized + SupportedValue + 'static, U: Vector<T> + VectorElemCount + 'static>(&mut self, vertex_data: U) {
        // Set the type if it wasn't set already
        if self.inner.is_none() { panic!() }
        // But if it was already set, we must check
        else if self.inner.as_ref().unwrap().1 != TypeId::of::<T>() { panic!(); }

        unsafe { self.add_unchecked(vertex_data); }
    }
    // Add a single custom vertex data, but without checking for type equality
    pub unsafe fn add_unchecked<T: Sized + SupportedValue + 'static, U: Vector<T> + 'static>(&mut self, vertex_data: U) {
        // Get the bytes first
        let bytes = vertex_data.to_native_bytes();
        let (vec, _, _) = self.inner.as_mut().unwrap();
        vec.extend_from_slice(bytes);
    }
    // Check if we have some custom data stored
    pub fn valid(&self) -> bool {
        self.inner.is_some()
    }
}