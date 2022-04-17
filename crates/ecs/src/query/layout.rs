use std::{ffi::c_void, ptr::NonNull, rc::Rc, cell::RefCell};

use crate::{registry, ComponentStateSet, LayoutAccess, PtrReader, ComponentStateRow, Entity, Mask, ArchetypeEntities, Archetype};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically

// This burns my eyeballs

pub trait QueryLayout<'a>
where
    Self: Sized,
{
    // Types
    type PtrTuple: 'static + Copy;

    // Get the pointer tuple from an archetype
    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple;

    // Get the final layout access masks
    fn combined() -> LayoutAccess;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

// Helper function to get the base pointer of a specific archetype
fn get_then_cast<'a, T: PtrReader<'a>>(archetype: &Archetype) -> NonNull<T::Component> {
    let ptr = registry::mask::<T::Component>();
    archetype.vectors[&ptr].1.cast()
}

impl<'a, A: PtrReader<'a>> QueryLayout<'a> for A {
    type PtrTuple = NonNull<A::Component>;

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        get_then_cast::<A>(archetype)
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        A::offset(tuple, bundle)
    }

    fn combined() -> LayoutAccess {
        A::access()
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (NonNull<A::Component>, NonNull<B::Component>);

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (get_then_cast::<A>(archetype), get_then_cast::<B>(archetype))
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle))
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access()
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>, C: PtrReader<'a>> QueryLayout<'a> for (A, B, C) {
    type PtrTuple = (NonNull<A::Component>, NonNull<B::Component>, NonNull<C::Component>);

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (get_then_cast::<A>(archetype), get_then_cast::<B>(archetype), get_then_cast::<C>(archetype))
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle), C::offset(tuple.2, bundle))
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access() | C::access()
    }
}
