use crate::{
    mask, Archetype, Component, ComponentTable, LayoutAccess, LinkError, Mask, MaskMap,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedBundle<'a>
where
    Self: Sized,
{
    // Mutable references to the required vectors stored within the archetypes
    type Storages: 'a;

    // Get the combined mask of the owned layout
    fn combined() -> Mask;

    // Check if this bundle is valid (a bundle is invalid if it has intersecting components)
    fn is_valid() -> bool;

    // Fetch the necessary storages from the archetype
    fn fetch(archetype: &'a mut Archetype) -> Self::Storages;

    // Push a new bundle into the storages
    fn push(storages: &mut Self::Storages, bundle: Self);
}

// Internal owned bundle that we will only use to create archetypes and their storeages
pub trait OwnedBundleTableAccessor: for<'a> OwnedBundle<'a> {
    // Get the default component tables that correspond to this bundle
    fn default_tables() -> MaskMap<Box<dyn ComponentTable>>;
    
    // Steal the underlying bundle from the given component tables
    fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Self;
    
    // Insert a new bundle into the given component tables
    fn push(storages: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Self;
}

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {
        // Implement the owned bundle for component sets
        impl<'a, $($name: Component),+> OwnedBundle<'a> for ($($name,)+) {
            type Storages = ($(&'a mut Vec<$name>),+);

            fn combined() -> Mask {
                ($(mask::<$name>())|+)
            }

            fn is_valid() -> bool {
                let anded = ($(mask::<$name>())&+);
                anded == Mask::zero()
            }

            fn fetch(archetype: &'a mut Archetype) -> Self::Storages {
                todo!()
                //assert!(Self::is_valid());
            }

            fn push(storages: &mut Self::Storages, bundle: Self) {

            }
        }

        // Implement the owned bundle table accessor for component sets
    };
}

tuple_impls! { C0, 2 }

/*
impl<'a, A: QueryItemReference<'a>> QueryLayout<'a> for A {
    type PtrTuple = A::Ptr;

    fn try_fetch_ptrs(archetype: &mut Archetype) -> Option<Self::PtrTuple> {
        A::try_fetch_ptr(archetype)
    }

    fn combined() -> LayoutAccess {
        A::access()
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
                ($($name::access())|+)
            }


            //    &A + &A      = valid
            // &mut A + &mut A = invalid
            //   &A + &mut A   = invalid
            //   &A + &mut B   = valid
            fn validate() -> bool {
                let combined = Self::combined();
                let intersecting = ($($name::access())&+);
                let self_intersect = combined.shared() & combined.unique() == Mask::zero();
                intersecting.unique() == Mask::zero() && self_intersect
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
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7, 8 }
*/
