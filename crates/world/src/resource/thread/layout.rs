use std::{slice::from_raw_parts, any::Any, sync::Arc};

use crate::SendPtr;

// Applied for immutable slice references of specific objects
// I had to separate this trait's method from the main's trait methods because poopy
pub trait RefSlice<'s: 'i, 'i> {
    type ItemRef: 'i;
    type OwnedItem: 'static;
    type Ptr: Any + Send + Sync + Copy + 'static;

    fn len(&self) -> Option<usize>;
    fn as_ptr(&self) -> Self::Ptr;
    unsafe fn from_raw_ptr(ptr: Self::Ptr, len: usize) -> Self;
    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr;
    unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRef;
}

// Simple RefSlice wrapper around normal slices
impl<'s: 'i, 'i, T: 'static> RefSlice<'s, 'i> for &'s [T] {
    type ItemRef = &'i T;
    type OwnedItem = T;
    type Ptr = SendPtr<T>;

    fn len(&self) -> Option<usize> {
        Some(<[T]>::len(self))
    }

    fn as_ptr(&self) -> Self::Ptr {
        <[T]>::as_ptr(self).into()
    }

    unsafe fn from_raw_ptr(ptr: Self::Ptr, len: usize) -> Self {
        from_raw_parts(ptr.into(), len)
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        let ptr: *const T = ptr.into();
        SendPtr::from(ptr.add(offset))
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRef {
        <[T]>::get_unchecked(self, index)
    }
}

// RefSlice wrapper around Option slices
impl<'s: 'i, 'i, T: 'static> RefSlice<'s, 'i> for Option<&'s [T]> {
    type ItemRef = Option<&'i T>;
    type OwnedItem = Option<T>;
    type Ptr = Option<SendPtr<T>>;

    fn len(&self) -> Option<usize> {
       None
    }

    fn as_ptr(&self) -> Self::Ptr {
        self.map(|s| s.as_ptr().into())
    }

    unsafe fn from_raw_ptr(ptr: Self::Ptr, len: usize) -> Self {
        ptr.map(|ptr| from_raw_parts(ptr.into(), len))        
    }

    unsafe fn offset_ptr(ptr: Self::Ptr, offset: usize) -> Self::Ptr {
        ptr.map(|ptr| {
            let ptr: *const T = ptr.into();
            SendPtr::from(ptr.add(offset))
        })
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRef {
        self.as_ref().map(|slice| <[T]>::get_unchecked(slice, index))
    }
}

// A ref layout contains one or more ref slices that will be iterated through
pub trait RefSliceTuple<'s: 'i, 'i>: 's + Sized {
    type PtrTuple: Any + Send + Sync + Copy + 'static;
    type ItemRefTuple: 'i;
    
    fn as_ptrs(&self) -> Self::PtrTuple;
    fn slice_tuple_len(&self) -> Option<usize>;
    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self;
    unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRefTuple;

    unsafe fn to_boxed_ptrs(self) -> Arc<dyn Any + Send + Sync + 'static> {
        Arc::new(self.as_ptrs())
    }

    unsafe fn from_boxed_ptrs(ptrs: Arc<dyn Any + Send + Sync + 'static>, length: usize, offset: usize) -> Option<Self> {
        let ptrs = ptrs.downcast::<Self::PtrTuple>().ok();
        ptrs.map(|ptrs| Self::from_ptrs(&*ptrs, length, offset))
    }
}

impl<'s: 'i, 'i, R: RefSlice<'s, 'i> + 's> RefSliceTuple<'s, 'i> for R {
    type PtrTuple = R::Ptr;
    type ItemRefTuple = R::ItemRef;

    fn as_ptrs(&self) -> Self::PtrTuple {
        self.as_ptr()
    }

    fn slice_tuple_len(&self) -> Option<usize> {
        self.len()
    }

    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self {
        Self::from_raw_ptr(R::offset_ptr(*ptrs, offset), length)
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRefTuple {
        <R as RefSlice<'s, 'i>>::get_unchecked(self, index)
    }
}