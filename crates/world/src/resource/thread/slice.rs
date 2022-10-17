use crate::{SendMutPtr, SendPtr};
use std::{
    any::Any,
    slice::{from_raw_parts, from_raw_parts_mut},
    sync::Arc,
};

// Implmented for immutable slice references
pub trait RefSlice<'i> {
    type ItemRef: 'i;
    type OwnedItem: 'static;
    type Ptr: Any + Send + Sync + Copy + 'static;

    fn len(&self) -> Option<usize>;
    fn as_ptr(&self) -> Self::Ptr;
    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self;
    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr;
    unsafe fn get_unchecked<'a: 'i>(&'a self, index: usize) -> Self::ItemRef;
}

// Implemented for mutable slice references
pub trait MutSlice<'i> {
    type ItemRef: 'i;
    type OwnedItem: 'static;
    type Ptr: Any + Send + Sync + Copy + 'static;

    fn len(&self) -> Option<usize>;
    fn as_ptr(&mut self) -> Self::Ptr;
    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self;
    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr;
    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRef;
}

// RefSlice impl for immutable slices
impl<'i, T: 'static> RefSlice<'i> for &[T] {
    type ItemRef = &'i T;
    type OwnedItem = T;
    type Ptr = SendPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn as_ptr(&self) -> Self::Ptr {
        <[T]>::as_ptr(self).into()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self {
        from_raw_parts(ptr.into(), len)
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        let ptr: *const T = ptr.into();
        SendPtr::from(ptr.add(offset))
    }

    unsafe fn get_unchecked<'a: 'i>(&'a self, index: usize) -> Self::ItemRef {
        <[T]>::get_unchecked(self, index)
    }
}

// RefSlice impl for Option immutable slices
impl<'i, T: 'static> RefSlice<'i> for Option<&[T]> {
    type ItemRef = Option<&'i T>;
    type OwnedItem = Option<T>;
    type Ptr = Option<SendPtr<T>>;

    fn len(&self) -> Option<usize> {
        None
    }

    fn as_ptr(&self) -> Self::Ptr {
        self.map(|s| s.as_ptr().into())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self {
        ptr.map(|ptr| from_raw_parts(ptr.into(), len))
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        ptr.map(|ptr| {
            let ptr: *const T = ptr.into();
            SendPtr::from(ptr.add(offset))
        })
    }

    unsafe fn get_unchecked<'a: 'i>(&'a self, index: usize) -> Self::ItemRef {
        self.as_ref()
            .map(|slice| <[T]>::get_unchecked(slice, index))
    }
}

// MutSlice impl for immutable slices
impl<'i, T: 'static> MutSlice<'i> for &[T] {
    type ItemRef = &'i T;
    type OwnedItem = T;
    type Ptr = SendPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn as_ptr(&mut self) -> Self::Ptr {
        <[T]>::as_ptr(self).into()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self {
        from_raw_parts(ptr.into(), len)
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        let ptr: *const T = ptr.into();
        SendPtr::from(ptr.add(offset))
    }

    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRef {
        <[T]>::get_unchecked(self, index)
    }
}

// MutSlice impl for mutable slices
impl<'i, T: 'static> MutSlice<'i> for &mut [T] {
    type ItemRef = &'i mut T;
    type OwnedItem = T;
    type Ptr = SendMutPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn as_ptr(&mut self) -> Self::Ptr {
        <[T]>::as_mut_ptr(self).into()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self {
        from_raw_parts_mut(ptr.into(), len)
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        let ptr: *mut T = ptr.into();
        SendMutPtr::from(ptr.add(offset))
    }

    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRef {
        <[T]>::get_unchecked_mut(self, index)
    }
}

// RefSlice impl for Option mutable slices
impl<'i, T: 'static> MutSlice<'i> for Option<&mut [T]> {
    type ItemRef = Option<&'i mut T>;
    type OwnedItem = Option<T>;
    type Ptr = Option<SendMutPtr<T>>;

    fn len(&self) -> Option<usize> {
        None
    }

    fn as_ptr(&mut self) -> Self::Ptr {
        self.as_mut().map(|s| s.as_mut_ptr().into())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self {
        ptr.map(|ptr| from_raw_parts_mut(ptr.into(), len))
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        ptr.map(|ptr| {
            let ptr: *mut T = ptr.into();
            SendMutPtr::from(ptr.add(offset))
        })
    }

    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRef {
        self.as_mut()
            .map(|slice| <[T]>::get_unchecked_mut(slice, index))
    }
}
