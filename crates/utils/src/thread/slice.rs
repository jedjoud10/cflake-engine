use crate::{SendMutPtr, SendPtr};
use std::{
    any::Any,
    slice::{from_raw_parts, from_raw_parts_mut},
};

// Implmented for immutable and mutable slice references
pub trait Slice<'i> {
    type Item: 'i;
    type OwnedItem: 'static + Sync + Send;
    type Ptr: Any + Send + Sync + Copy + 'static;

    fn len(&self) -> Option<usize>;
    fn is_empty(&self) -> Option<bool>;
    fn as_ptr(&mut self) -> Self::Ptr;
    unsafe fn from_raw_parts(ptr: Self::Ptr, len: usize) -> Self;
    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr;
    unsafe fn get_unchecked<'a: 'i>(
        &'a mut self,
        index: usize,
    ) -> Self::Item;
}

// Slice impl for immutable slices
impl<'i, T: 'static + Sync + Send> Slice<'i> for &[T] {
    type Item = &'i T;
    type OwnedItem = T;
    type Ptr = SendPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn is_empty(&self) -> Option<bool> {
        Some(<[T]>::is_empty(self))
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

    unsafe fn get_unchecked<'a: 'i>(
        &'a mut self,
        index: usize,
    ) -> Self::Item {
        <[T]>::get_unchecked(self, index)
    }
}

// Slice impl for Option immutable slices
impl<'i, T: 'static + Sync + Send> Slice<'i> for Option<&[T]> {
    type Item = Option<&'i T>;
    type OwnedItem = Option<T>;
    type Ptr = Option<SendPtr<T>>;

    fn len(&self) -> Option<usize> {
        self.map(|s| <[T]>::len(s))
    }

    fn is_empty(&self) -> Option<bool> {
        self.map(|s| <[T]>::is_empty(s))
    }

    fn as_ptr(&mut self) -> Self::Ptr {
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

    unsafe fn get_unchecked<'a: 'i>(
        &'a mut self,
        index: usize,
    ) -> Self::Item {
        self.as_ref()
            .map(|slice| <[T]>::get_unchecked(slice, index))
    }
}

// Slice impl for mutable slices
impl<'i, T: 'static + Sync + Send> Slice<'i> for &mut [T] {
    type Item = &'i mut T;
    type OwnedItem = T;
    type Ptr = SendMutPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn is_empty(&self) -> Option<bool> {
        Some(<[T]>::is_empty(self))
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

    unsafe fn get_unchecked<'a: 'i>(
        &'a mut self,
        index: usize,
    ) -> Self::Item {
        <[T]>::get_unchecked_mut(self, index)
    }
}

// RefSlice impl for Option mutable slices
impl<'i, T: 'static + Sync + Send> Slice<'i> for Option<&mut [T]> {
    type Item = Option<&'i mut T>;
    type OwnedItem = Option<T>;
    type Ptr = Option<SendMutPtr<T>>;

    fn len(&self) -> Option<usize> {
        self.as_ref().map(|s| <[T]>::len(s))
    }

    fn is_empty(&self) -> Option<bool> {
        self.as_ref().map(|s| <[T]>::is_empty(s))
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

    unsafe fn get_unchecked<'a: 'i>(
        &'a mut self,
        index: usize,
    ) -> Self::Item {
        self.as_mut()
            .map(|slice| <[T]>::get_unchecked_mut(slice, index))
    }
}
