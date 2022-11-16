use super::slice::*;

use std::any::Any;

// A ref/mut layout contains one or more ref/mut slices that will be iterated through in other threads
pub trait SliceTuple<'i>: Sized {
    type PtrTuple: Any + Send + Sync + Copy + 'static;
    type OwnedTuple: 'static + Send + Sync;
    type ItemTuple;

    // Into ptrs, from ptrs, length, and get
    fn as_ptrs(&mut self) -> Self::PtrTuple;
    fn slice_tuple_len(&self) -> Option<usize>;
    unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self;
    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemTuple;
}

// Implement the ref slice tuple for immutable slices
impl<'i, R: Slice<'i>> SliceTuple<'i> for R {
    type PtrTuple = R::Ptr;
    type ItemTuple = R::Item;
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

    unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemTuple {
        <R as Slice<'i>>::get_unchecked(self, index)
    }
}
