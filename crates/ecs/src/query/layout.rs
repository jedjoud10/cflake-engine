use std::ptr::NonNull;

use crate::{Archetype, LayoutAccess, PtrReader};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
// This burns my eyeballs

pub trait QueryLayout<'a>
where
    Self: Sized,
{
    // A tuple that contains the underlying base pointers for the components
    type PtrTuple: 'static + Copy;

    // Tuple that contains the components. This is mostly used to check if the query layout can be safely sent to another thread for parallel execution
    type Tuple: 'static;

    // Get the pointer tuple from an archetype
    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple;

    // Get the final layout access masks
    fn combined() -> LayoutAccess;

    // This must return "false" if any of the items have intersecting masks
    fn validate() -> bool;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

impl<'a, A: PtrReader<'a>> QueryLayout<'a> for A {
    type PtrTuple = NonNull<A::Item>;
    type Tuple = A::Item;

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        A::fetch(archetype)
    }

    fn combined() -> LayoutAccess {
        A::access()
    }

    fn validate() -> bool {
        true
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        A::offset(tuple, bundle)
    }

}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (NonNull<A::Item>, NonNull<B::Item>);
    type Tuple = (A::Item, B::Item);

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (A::fetch(archetype), B::fetch(archetype))
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access()
    }

    fn validate() -> bool {
        (A::access() & B::access()) == LayoutAccess::none()
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle))
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>, C: PtrReader<'a>> QueryLayout<'a> for (A, B, C) {
    type PtrTuple = (NonNull<A::Item>, NonNull<B::Item>, NonNull<C::Item>);
    type Tuple = (A::Item, B::Item, C::Item);

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (A::fetch(archetype), B::fetch(archetype), C::fetch(archetype))
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access() | C::access()
    }

    fn validate() -> bool {
        (A::access() & B::access() & C::access()) == LayoutAccess::none()
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle), C::offset(tuple.2, bundle))
    }
}
