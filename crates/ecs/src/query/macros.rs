use crate::{
    mask, Archetype, Component, ComponentTable, LayoutAccess, LinkError, Mask, MaskMap, OwnedBundle, OwnedBundleAnyTableAccessor, RefQueryItem, RefQueryLayout, MutQueryLayout, MutQueryItem
};

use seq_macro::seq;
use casey::lower;

// Implement the owned bundle for single component
impl<'a, T: Component> OwnedBundle<'a> for T {
    type Storages = &'a mut Vec<T>;

    fn combined() -> Mask {
        mask::<T>()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages> {
        archetype.table_mut::<T>()
    }

    fn push(storages: &mut Self::Storages, bundle: Self) {
        storages.push(bundle)
    }
}

// Implement the owned bundle table accessor for single component
impl<T: Component + for<'a> OwnedBundle<'a>> OwnedBundleAnyTableAccessor for T {
    fn default_tables() -> MaskMap<Box<dyn ComponentTable>> {
        let boxed: Box<dyn ComponentTable> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Option<Self> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Some(vec.swap_remove(index))
    }

    fn push(tables: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Option<()> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        vec.push(bundle);

        Some(())
    }
}

// Implementation of ref query layout for single component
impl<'a, T: RefQueryItem<'a>> RefQueryLayout<'a> for T {
    type Slices = &'a [T::Item];

    fn prepare(archetype: &'a Archetype) -> Option<Self::Slices> {
        T::get(archetype)
    }

    fn is_valid() -> bool {
        true
    }

    fn read(cache: &'a Self::Slices, i: usize) -> Self {
        T::read(cache, i)
    }
}

// Implementation of mut query layout for single component 
impl<'a, T: MutQueryItem<'a>> MutQueryLayout<'a> for T {
    type Slices = &'a mut [T::Item];

    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Slices> {
        T::get(archetype)
    }

    fn is_valid() -> bool {
        true
    }

    fn read(cache: &'a mut Self::Slices, i: usize) -> Self {
        T::read(cache, i)
    }
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
                ($(mask::<$name>())&+) == Mask::zero()
            }

            fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages> {
                todo!()
            }

            fn push(storages: &mut Self::Storages, bundle: Self) {
                seq!(N in 0..$max {
                    let vec = &mut storages.N;
                    vec.push(bundle.N);
                });
            }
        }

        // Implement the owned bundle table accessor for component sets
        impl<$($name: Component),+> OwnedBundleAnyTableAccessor for ($($name,)+) {
            fn default_tables() -> MaskMap<Box<dyn ComponentTable>> {
                let mut map = MaskMap::<Box<dyn ComponentTable>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }

            fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Option<Self> {
                // TODO: Remove this hack and actually figure out something gud
                seq!(N in 0..$max {
                    let boxed = tables.get_mut(&mask::<C~N>())?;
                    let vec = boxed.as_any_mut().downcast_mut::<Vec<C~N>>().unwrap();
                    let c~N: C~N = vec.swap_remove(index);
                });

                Some(($(
                    lower!($name)
                ),+,))
            }

            fn push(tables: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Option<()> {
                seq!(N in 0..$max {
                    let boxed = tables.get_mut(&mask::<C~N>())?;
                    let vec = boxed.as_any_mut().downcast_mut::<Vec<C~N>>().unwrap();
                    vec.push(bundle.N);
                });

                Some(())
            }
        }

        // Implement the mutable query layout for the tuples
        impl<'a, $($name: MutQueryItem<'a>),+> MutQueryLayout<'a> for ($($name,)+) {
            type Slices = ($(&'a mut [$name::Item]),+);
            fn prepare(archetype: &'a mut Archetype) -> Option<Self::Slices> {
                todo!()
            }

            fn is_valid() -> bool {
                let intersecting = ($(mask::<$name::Item>())&+);
                let combined = ($($name::access())|+);

                let a = intersecting == Mask::zero();
                let b = combined.shared() & combined.unique() == Mask::zero();
                a && b
            }
            
            fn read(slices: &'a mut Self::Slices, i: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::read(slices.N, i);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        // Implement the immutable query layout for the tuples
        impl<'a, $($name: RefQueryItem<'a>),+> RefQueryLayout<'a> for ($($name,)+) {
            type Slices = ($(&'a [$name::Item]),+);
            fn prepare(archetype: &'a Archetype) -> Option<Self::Slices> {
                todo!()
            }

            fn is_valid() -> bool {
                ($(mask::<$name::Item>())&+) == Mask::zero()
            }
            
            fn read(slices: &'a Self::Slices, i: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::read(slices.N, i);
                });

                ($(
                    lower!($name)
                ),+,)
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
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }