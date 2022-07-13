use std::ptr::NonNull;

use crate::{Archetype, Component, LayoutAccess, LinkError, LinkModifier, Mask};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems (&T, &mut T, &Entity)
pub trait QueryLayout<'a>
where
    Self: Sized + 'a,
{
    // A tuple that contains the underlying base pointers for the components
    type PtrTuple: 'static + Copy;

    // Get the pointer tuple from an archetype
    // This assumes that the archetype contains said pointers
    fn try_fetch_ptrs(archetype: &mut Archetype) -> Option<Self::PtrTuple>;

    // Get the final layout access masks
    fn combined() -> LayoutAccess;

    // This must return "false" if any of the items have intersecting masks
    fn validate() -> bool;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

// A view layout for queries that are not mutable, and that only use &T and &Entity
pub trait ViewLayout<'a>
where
    Self: Sized + 'a,
{
    // A tuple that contains the underlying base pointers for the components
    type PtrTuple: 'static + Copy;

    // Get the pointer tuple from an archetype
    // This assumes that the archetype contains said pointers
    unsafe fn try_fetch_ptrs(archetype: &Archetype) -> Option<Self::PtrTuple>;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedLayout
where
    Self: Sized,
{
    // Get the combined mask of the owned layout
    fn mask() -> Mask;

    // Consume the tuple and insert the components using a link modifier
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError>;
}
