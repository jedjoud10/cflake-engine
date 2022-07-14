use std::{ptr::NonNull, mem::MaybeUninit};
use crate::{Archetype, Component, LayoutAccess, LinkError, LinkModifier, Mask, QueryItemReference, ViewItemReference, mask};
use seq_macro::seq;
use casey::lower;

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
    fn validate() -> bool {
        todo!()
    }

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

    // Get the final layout access mask
    fn combined() -> Mask;

    // Get the pointer tuple from an archetype
    // This assumes that the archetype contains said pointers
    unsafe fn try_fetch_ptrs(archetype: &Archetype) -> Option<Self::PtrTuple>;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self;
}

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedComponentLayout
where
    Self: Sized,
{
    // Get the combined mask of the owned layout
    fn mask() -> Mask;

    // Consume the tuple and insert the components using a link modifier
    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError>;
}

impl<'a, A: QueryItemReference<'a>> QueryLayout<'a> for A {
    type PtrTuple = A::Ptr;

    fn try_fetch_ptrs(archetype: &mut Archetype) -> Option<Self::PtrTuple> {
        A::try_fetch_ptr(archetype)
    }

    fn combined() -> LayoutAccess {
        A::read_write_access()
    }

    fn validate() -> bool {
        true
    }

    unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self {
        A::as_self(tuple, bundle)
    }
}

impl<'a, A: ViewItemReference<'a>> ViewLayout<'a> for A {
    type PtrTuple = *const A::Item;

    fn combined() -> Mask {
        A::read_mask()
    }

    unsafe fn try_fetch_ptrs(archetype: &Archetype) -> Option<Self::PtrTuple> {
        A::try_fetch_ptr(archetype)
    }

    unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self {
        A::as_ref(tuple, bundle)
    }
}

impl<A: Component> OwnedComponentLayout for A {
    fn mask() -> Mask {
        mask::<A>()
    }

    fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
        modifier.insert(self)
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {
        // Implement the mutable query layout trait
        impl<'a, $($name: QueryItemReference<'a>),+> QueryLayout<'a> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);
        
            fn try_fetch_ptrs(archetype: &mut Archetype) -> Option<Self::PtrTuple> {
                let data = ($($name::try_fetch_ptr(archetype)?,)+);
                Some(data)
            }
        
            fn combined() -> LayoutAccess {
                ($($name::read_write_access())|+)
            }
        
            unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N: C~N = C~N::as_self(tuple.N, bundle);
                });
                
                ($(
                    lower!($name)
                ),+,)
            }
        }

        // Implement the immutable view query layout trait
        impl<'a, $($name: ViewItemReference<'a>),+> ViewLayout<'a> for ($($name,)+) {
            type PtrTuple = ($(*const $name::Item),+);
        
            fn combined() -> Mask {
                ($($name::read_mask())|+)
            }
        
            unsafe fn try_fetch_ptrs(archetype: &Archetype) -> Option<Self::PtrTuple> {
                let data = ($($name::try_fetch_ptr(archetype)?,)+);
                Some(data)
            }
        
            unsafe fn read_as_layout_at(tuple: Self::PtrTuple, bundle: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N: C~N = C~N::as_ref(tuple.N, bundle);
                });
                
                ($(
                    lower!($name)
                ),+,)
            }
        }

        // Implement the owned layout for component sets
        impl<$($name: Component),+> OwnedComponentLayout for ($($name,)+) {
            fn mask() -> Mask {
                ($(mask::<$name>())|+)
            }
        
            fn insert(self, modifier: &mut LinkModifier) -> Result<(), LinkError> {
                seq!(N in 0..$max {
                    modifier.insert(self.N)?;
                });
                Ok(())
            }
        }
    };    
}

tuple_impls! { C0 C1, 2 }
tuple_impls! { C0 C1 C2, 3 }
tuple_impls! { C0 C1 C2 C3, 4 }
tuple_impls! { C0 C1 C2 C3 C4, 5 }
tuple_impls! { C0 C1 C2 C3 C4 C5, 6 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6, 7 }