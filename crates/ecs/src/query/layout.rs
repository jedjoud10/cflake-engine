use std::ptr::NonNull;

use crate::{Archetype, Component, LayoutAccess, LinkError, LinkModifier, Mask, PtrReader};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a>
where
    Self: Sized,
{
    // A tuple that contains the underlying base pointers for the components
    type PtrTuple: 'static + Copy;

    // Tuple that contains the components. This is mostly used to check if the query layout can be safely sent to another thread for parallel execution
    type Tuple: 'static;

    // Get the pointer tuple from an archetype
    // This assumes that the archetype contains said pointers
    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple;

    // Get the final layout access masks
    fn combined() -> LayoutAccess;

    // This must return "false" if any of the items have intersecting masks
    fn validate() -> bool;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedLayout
where
    Self: Sized,
{
    // Consume the tuple and insert the components using a link modifier
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError>;
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
        (
            A::fetch(archetype),
            B::fetch(archetype),
            C::fetch(archetype),
        )
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access() | C::access()
    }

    fn validate() -> bool {
        (A::access() & B::access() & C::access()) == LayoutAccess::none()
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (
            A::offset(tuple.0, bundle),
            B::offset(tuple.1, bundle),
            C::offset(tuple.2, bundle),
        )
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>, C: PtrReader<'a>, D: PtrReader<'a>> QueryLayout<'a>
    for (A, B, C, D)
{
    type PtrTuple = (
        NonNull<A::Item>,
        NonNull<B::Item>,
        NonNull<C::Item>,
        NonNull<D::Item>,
    );
    type Tuple = (A::Item, B::Item, C::Item, D::Item);

    fn get_base_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (
            A::fetch(archetype),
            B::fetch(archetype),
            C::fetch(archetype),
            D::fetch(archetype),
        )
    }

    fn combined() -> LayoutAccess {
        A::access() | B::access() | C::access() | D::access()
    }

    fn validate() -> bool {
        (A::access() & B::access() & C::access() & D::access()) == LayoutAccess::none()
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self {
        (
            A::offset(tuple.0, bundle),
            B::offset(tuple.1, bundle),
            C::offset(tuple.2, bundle),
            D::offset(tuple.3, bundle),
        )
    }
}

impl<T: Component> OwnedLayout for T {
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
        modifier.insert(self)
    }
}

impl<A: Component, B: Component> OwnedLayout for (A, B) {
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
        modifier.insert(self.0)?;
        modifier.insert(self.1)
    }
}

impl<A: Component, B: Component, C: Component> OwnedLayout for (A, B, C) {
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
        modifier.insert(self.0)?;
        modifier.insert(self.1)?;
        modifier.insert(self.2)
    }
}

impl<A: Component, B: Component, C: Component, D: Component> OwnedLayout for (A, B, C, D) {
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
        modifier.insert(self.0)?;
        modifier.insert(self.1)?;
        modifier.insert(self.2)?;
        modifier.insert(self.3)?;
        Ok(())
    }
}
