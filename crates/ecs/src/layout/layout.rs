use std::alloc::Layout;

use crate::{Archetype, QueryItem, LayoutAccess, Mask};

// A query layout is a combination of multiple query items 
pub trait QueryLayout<'s, 'i> {
    type SliceTuple: 's;

    // Get the number of query items that this layout contains
    fn items() -> usize;

    // Get the debug name of a specific query item
    fn name(index: usize) -> Option<&'static str>;

    // Get the layout accesses of a specific component
    fn access(index: usize) -> Option<LayoutAccess>;
    
    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;
    
    // This checks if the layout is valid (no collisions, no ref-mut collisions)
    fn is_valid() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        let a = combined.shared() & combined.unique() == Mask::zero();
        dbg!(a);
        let mut_items = (0..Self::items()).into_iter().filter(|i| Self::access(*i).unwrap().unique() != Mask::zero()).count() as u64;
        let enabled: u64 = combined.unique().into();
        let b = mut_items == enabled.count_ones() as u64;  
        dbg!(b);
        a && b
    }

    // Check if the query layout contains any mutable items
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique() != Mask::zero()
    }

    // Check if the layout is valid for usage for any case
    /*
    fn is_valid() -> Result<(), QueryError> {
        let combined = Self::reduce(|a, b| a | b);
        
        /*
        // Check if we have any missing components from the archetype
        if combined.both() & archetype.mask() != combined.both()  {
            let on = LayoutAccess::new(archetype.mask(), archetype.mask());
            let diff = (combined & on) ^ combined;
            let index = (0..Self::items()).position(|i| {
                let local = Self::access(i).unwrap();
                local & diff == local
            }).unwrap();
            return Err(QueryError::MissingArchetypeTable(Self::name(index).unwrap()));
        }
        */

        // Check if we have any components that have duplicate mutable access
        // TODO: Fix this
        let converted: u64 = combined.unique().into();
        if converted.count_ones() != (Self::items() as u32) {
            let name = Self::name(converted.trailing_zeros() as usize);
            return Err(QueryError::MultipleMutableAccess(name.unwrap()));
        }        
    
        // Check if we have any components that have simultaneous mutable and immutable access
        if combined.shared() & combined.unique() == Mask::zero() {
            let converted: u64 = (combined.shared() & combined.unique()).into();
            let index = converted.trailing_zeros() as usize;
            let name = Self::name(index).unwrap();
            return Err(QueryError::SimultaneousMutRefAccess(name));
        }


        Ok(())        
    }
    */

    /*
    // Get the query layout slice tuple from the corresponding archetypes
    fn slices_from_archetype(archetype: &Archetype) -> Result<Self::SliceTuple, QueryError> {
        let index = (0..Self::items())
            .position(|i| 
                Self::access(i)
                    .unwrap()
                    .unique() != Mask::zero()
        );

        let err = if let Some(index) = index {
            Err(QueryError::MutableAccessWhilstView(<Self as QueryLayout<'s, 'i>>::name(index).unwrap()))
        } else { 
            Self::is_valid(archetype)
        };

        err.map(|_| unsafe { Self::slices_from_archetype_unchecked(archetype) })
    }

    fn slices_from_mut_archetype(archetype: &mut Archetype) -> Result<Self::SliceTuple, QueryError> {
        Self::is_valid(archetype).map(|_| unsafe { Self::slices_from_mut_archetype_unchecked(archetype) })
    }
    */

    // Get the query layout slice tuple from the corresponding archetypes without checking for safety
    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self::SliceTuple;
    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::SliceTuple;

    // Read from the valid slice tuple
    unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self;
}

impl<'s: 'i, 'i, I: QueryItem<'s, 'i>> QueryLayout<'s, 'i> for I {
    type SliceTuple = I::Slice;

    fn items() -> usize {
        1
    }

    fn name(index: usize) -> Option<&'static str> {
        (index == 0).then(|| I::name()) 
    }

    fn access(index: usize) -> Option<LayoutAccess> {
        (index == 0).then(|| I::access()) 
    }

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access()).into_iter().reduce(lambda).unwrap()
    }

    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self::SliceTuple {
        let ptr = I::ptr_from_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::SliceTuple {
        let ptr = I::ptr_from_mut_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self {
        <I as QueryItem<'s, 'i>>::get_unchecked(slice, index)
    }
}