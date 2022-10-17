use super::slice::*;
use crate::SendPtr;
use seq_macro::seq;
use std::{any::Any, slice::from_raw_parts, sync::Arc};

// A ref layout contains one or more ref slices that will be iterated through in other threads
pub trait RefSliceTuple<'i>: Sized {
    type PtrTuple: Any + Send + Sync + Copy + 'static;
    type OwnedTuple: 'static;
    type ItemRefTuple;

    // Into ptrs, from ptrs, length, and get
    fn as_ptrs(&self) -> Self::PtrTuple;
    fn slice_tuple_len(&self) -> Option<usize>;
    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self;
    unsafe fn get_unchecked<'a: 'i>(&'a self, index: usize) -> Self::ItemRefTuple;

    // Converts the ptr type into boxed pointers
    unsafe fn to_boxed_ptrs(self) -> Arc<dyn Any + Send + Sync + 'static> {
        Arc::new(self.as_ptrs())
    }

    // Tries to convert base boxed pointers into Self
    unsafe fn from_boxed_ptrs(
        ptrs: Arc<dyn Any + Send + Sync + 'static>,
        length: usize,
        offset: usize,
    ) -> Option<Self> {
        let ptrs = ptrs.downcast::<Self::PtrTuple>().ok();
        ptrs.map(|ptrs| Self::from_ptrs(&*ptrs, length, offset))
    }
}

// A mut layout contains one or more mut slices that will be iterated through in other threads
pub trait MutSliceTuple<'i>: Sized {
    type PtrTuple: Any + Send + Sync + Copy + 'static;
    type OwnedTuple: 'static;
    type ItemRefTuple: 'i;

    // Into ptrs, from ptrs, length, and get
    fn as_ptrs(&mut self) -> Self::PtrTuple;
    fn slice_tuple_len(&self) -> Option<usize>;
    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self;
    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRefTuple;

    // Converts the ptr type into boxed pointers
    unsafe fn to_boxed_ptrs(mut self) -> Arc<dyn Any + Send + Sync + 'static> {
        Arc::new(self.as_ptrs())
    }

    // Tries to convert base boxed pointers into Self
    unsafe fn from_boxed_ptrs(
        ptrs: Arc<dyn Any + Send + Sync + 'static>,
        length: usize,
        offset: usize,
    ) -> Option<Self> {
        let ptrs = ptrs.downcast::<Self::PtrTuple>().ok();
        ptrs.map(|ptrs| Self::from_ptrs(&*ptrs, length, offset))
    }
}

// Implement the ref slice tuple for immutable slices
impl<'i, R: RefSlice<'i>> RefSliceTuple<'i> for R {
    type PtrTuple = R::Ptr;
    type ItemRefTuple = R::ItemRef;
    type OwnedTuple = R::OwnedItem;

    fn as_ptrs(&self) -> Self::PtrTuple {
        self.as_ptr()
    }

    fn slice_tuple_len(&self) -> Option<usize> {
        self.len()
    }

    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self {
        Self::from_raw_parts(R::offset_ptr(*ptrs, offset), length)
    }

    unsafe fn get_unchecked<'a: 'i>(&'a self, index: usize) -> Self::ItemRefTuple {
        <R as RefSlice<'i>>::get_unchecked(self, index)
    }
}

// Implement the mut slice tuple for mutable slices
impl<'i, R: MutSlice<'i>> MutSliceTuple<'i> for R {
    type PtrTuple = R::Ptr;
    type ItemRefTuple = R::ItemRef;
    type OwnedTuple = R::OwnedItem;

    fn as_ptrs(&mut self) -> Self::PtrTuple {
        self.as_ptr()
    }

    fn slice_tuple_len(&self) -> Option<usize> {
        self.len()
    }

    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self {
        Self::from_raw_parts(R::offset_ptr(*ptrs, offset), length)
    }

    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemRefTuple {
        <R as MutSlice<'i>>::get_unchecked(self, index)
    }
}
