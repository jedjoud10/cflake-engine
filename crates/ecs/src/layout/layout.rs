use crate::{Archetype, QueryItem, LayoutAccess, QueryValidityError, Mask};

// A query layout is a combination of multiple query items 
pub trait QueryLayout<'s, 'i> {
    type PtrTuple: 'static + Copy;
    type SliceTuple: 's;

    // Get the number of query items that this layout contains
    fn items() -> usize;

    // Get the debug name of a specific query item
    fn name(index: usize) -> &'static str;

    // Check if the layout is valid for usage
    fn is_valid() -> Result<(), QueryValidityError> {
        let combined = Self::fold(|a, b| a | b);
        
        // Check if we have any components that have duplicate mutable access
        let converted: u64 = combined.unique().into();
        if converted.count_ones() != (Self::items() as u32) {
            let name = Self::name(converted.trailing_zeros() as usize);
            return Err(QueryValidityError::MultipleMutableAccess(name));
        }        
    
        // Check if we have any components that have simultaneous mutable and immutable access
        if combined.shared() & combined.unique() == Mask::zero() {
            let converted: u64 = (combined.shared() & combined.unique()).into();
            let index = converted.trailing_zeros() as usize;
            return Err(QueryValidityError::SimultaneousMutRefAccess(Self::name(index)));
        }

        Ok(())        
    }

    // Get a combined layout access mask by running a lambda on each layout
    fn fold(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // Get the query layout pointer tuple from the corresponding archetypes
    fn ptrs_from_archetype(archetype: &Archetype) -> Self::PtrTuple;
    fn ptrs_from_mut_archetype(archetype: &mut Archetype) -> Self::PtrTuple;

    // Convert the pointer tuple into a slice tuple, and read from said slice tuple
    unsafe fn from_raw_parts(tuple: Self::PtrTuple, length: usize) -> Self::SliceTuple;
    unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self;
}

impl<'s: 'i, 'i, I: QueryItem<'s, 'i>> QueryLayout<'s, 'i> for I {
    type PtrTuple = I::Ptr;
    type SliceTuple = I::Slice;

    fn items() -> usize {
        1
    }

    fn is_valid() -> Result<(), QueryValidityError> {
        Ok(())
    }

    fn name(index: usize) -> &'static str {
        if index == 0 {
            I::name()
        } else {
            panic!()
        }
    }

    fn fold(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        [I::access()].into_iter().fold(LayoutAccess::none(), lambda)
    }

    fn ptrs_from_archetype(archetype: &Archetype) -> Self::PtrTuple {
        I::ptr_from_archetype(archetype)
    }

    fn ptrs_from_mut_archetype(archetype: &mut Archetype) -> Self::PtrTuple {
        I::ptr_from_mut_archetype(archetype)
    }

    unsafe fn from_raw_parts(tuple: Self::PtrTuple, length: usize) -> Self::SliceTuple {
        <I as QueryItem<'s, 'i>>::from_raw_parts(tuple, length)
    }

    unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self {
        <I as QueryItem<'s, 'i>>::get_unchecked(slice, index)
    }
}