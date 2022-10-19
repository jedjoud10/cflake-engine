use std::slice;

use crate::{
    mask, Archetype, Bundle, Component, ComponentTable, LayoutAccess, Mask, MaskHashMap,
};

use casey::lower;
use seq_macro::seq;

/*

// Impl of mut query item for &T
impl<'s: 'l, 'l, T: Component> MutQueryItem<'s, 'l> for &'l T {
    type Item = T;
    type Ptr = *const T;
    type Slice = &'s [T];

    fn access(m: Mask) -> Option<LayoutAccess> {
        let cm = mask::<T>();
        m.contains(cm)
            .then_some(LayoutAccess::new(cm, Mask::zero()))
    }

    fn prepare<'ar: 's>(archetype: &'ar mut Archetype) -> Option<Self::Ptr> {
        archetype.table_mut::<T>().map(|vec| vec.as_ptr())
    }

    unsafe fn into_slice(ptr: Self::Ptr, length: usize) -> Self::Slice {
        slice::from_raw_parts(ptr, length)
    }

    fn read_from_slice(slice: &mut Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}

// Impl of mut query item for Option<&T>
impl<'s: 'l, 'l, T: Component> MutQueryItem<'s, 'l> for Option<&'l T> {
    type Item = T;
    type Ptr = Option<*const T>;
    type Slice = Option<&'s [T]>;

    fn access(m: Mask) -> Option<LayoutAccess> {
        Some(LayoutAccess::new(mask::<T>() & m, Mask::zero()))
    }

    fn prepare<'ar: 's>(archetype: &'ar mut Archetype) -> Option<Self::Ptr> {
        Some(archetype.table::<T>().map(|vec| vec.as_ptr()))
    }

    unsafe fn into_slice(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| slice::from_raw_parts(ptr, length))
    }

    fn read_from_slice(slice: &mut Self::Slice, index: usize) -> Self {
        slice.map(|slice| &slice[index])
    }
}

// Impl of mut query item for &mut T
impl<'s: 'l, 'l, T: Component> MutQueryItem<'s, 'l> for &'l mut T {
    type Item = T;
    type Ptr = *mut T;
    type Slice = &'s mut [T];

    fn access(m: Mask) -> Option<LayoutAccess> {
        let cm = mask::<T>();
        m.contains(cm)
            .then_some(LayoutAccess::new(Mask::zero(), cm))
    }

    fn prepare<'ar: 's>(archetype: &'ar mut Archetype) -> Option<Self::Ptr> {
        archetype.table_mut::<T>().map(|vec| vec.as_mut_ptr())
    }

    unsafe fn into_slice(ptr: Self::Ptr, length: usize) -> Self::Slice {
        slice::from_raw_parts_mut(ptr, length)
    }

    fn read_from_slice(slice: &mut Self::Slice, index: usize) -> Self {
        &mut slice[index]
    }
}

// Impl of mut query item for Option<&mut T>
impl<'s: 'l, 'l, T: Component> MutQueryItem<'s, 'l> for Option<&'l mut T> {
    type Item = T;
    type Ptr = Option<*mut T>;
    type Slice = Option<&'s mut [T]>;

    fn access(m: Mask) -> Option<LayoutAccess> {
        Some(LayoutAccess::new(Mask::zero(), m & mask::<T>()))
    }

    fn prepare<'ar: 's>(archetype: &'ar mut Archetype) -> Option<Self::Ptr> {
        Some(archetype.table_mut::<T>().map(|vec| vec.as_mut_ptr()))
    }

    unsafe fn into_slice(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| slice::from_raw_parts_mut(ptr, length))
    }

    fn read_from_slice(slice: &mut Self::Slice, index: usize) -> Self {
        slice.map(|slice| &mut slice[index])
    }
}

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

    fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>> {
        let boxed: Box<dyn ComponentTable> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentTable>>,
        index: usize,
    ) -> Option<Self> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Some(vec.swap_remove(index))
    }
}

impl<T: Component> Bundle for T {}

// Impl of ref query layout for single component
impl<'a, T: RefQueryItem<'a>> RefQueryLayout<'a> for T {
    type PtrTuple = T::Ptr;

    fn is_valid() -> bool {
        true
    }

    fn prepare(archetype: &Archetype) -> Option<Self::PtrTuple> {
        <T as RefQueryItem<'a>>::prepare(archetype)
    }

    unsafe fn read(ptr: Self::PtrTuple, i: usize) -> Self {
        <T as RefQueryItem<'a>>::read(ptr, i)
    }

    fn access(m: Mask) -> Option<LayoutAccess> {
        <T as RefQueryItem<'a>>::access(m)
    }
}

// Impl of mut query layout for single component
impl<'s: 'l, 'l, T: MutQueryItem<'s, 'l>> MutQueryLayout<'s, 'l> for T {
    type SliceTuple = T::Slice;

    fn is_valid() -> bool {
        true
    }

    fn prepare<'ar: 's>(archetype: &'ar mut Archetype) -> Option<Self::SliceTuple> {
        let ptr = <T as MutQueryItem<'ar, 's, 'l>>::prepare(archetype)?;
        Some(unsafe { <T as MutQueryItem<'ar, 's, 'l>>::into_slice(ptr, archetype.len()) })
    }

    fn read(tuple: &mut Self::SliceTuple, index: usize) -> Self {
        T::read_from_slice(tuple, index)
    }

    fn access(m: Mask) -> Option<LayoutAccess> {
        <T as MutQueryItem<'s, 'l>>::access(m)
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
                assert!(Self::is_valid());
                // TODO: Remove this hack
                seq!(N in 0..$max {
                    let table = archetype.table_mut::<C~N>()?;
                    let ptr = table as *mut Vec<C~N>;
                    let c~N = unsafe { &mut *ptr };
                });

                Some(($(
                    lower!($name)
                ),+,))
            }

            fn push(storages: &mut Self::Storages, bundle: Self) {
                seq!(N in 0..$max {
                    let vec = &mut storages.N;
                    vec.push(bundle.N);
                });
            }

            fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>> {
                let mut map = MaskHashMap::<Box<dyn ComponentTable>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }

            fn try_swap_remove(tables: &mut MaskHashMap<Box<dyn ComponentTable>>, index: usize) -> Option<Self> {
                seq!(N in 0..$max {
                    let boxed = tables.get_mut(&mask::<C~N>())?;
                    let vec = boxed.as_any_mut().downcast_mut::<Vec<C~N>>().unwrap();
                    let c~N: C~N = vec.swap_remove(index);
                });

                Some(($(
                    lower!($name)
                ),+,))
            }
        }

        // Simple trait wrapper for these types as well
        impl<'a, $($name: Component),+> Bundle for ($($name,)+) {}

        // Implement the mutable query layout for the tuples
        impl<'a, $($name: MutQueryItem<'a>),+> MutQueryLayout<'a> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);

            fn is_valid() -> bool {
                let intersecting = ($(mask::<$name::Component>())&+);
                let combined = ($($name::access(Mask::all()).unwrap())|+);

                let a = intersecting == Mask::zero();
                let b = combined.shared() & combined.unique() == Mask::zero();
                a && b
            }

            fn access(m: Mask) -> Option<LayoutAccess> {
                Some(($($name::access(m)?)|+))
            }

            fn prepare(archetype: &mut Archetype) -> Option<Self::PtrTuple> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    let c~N = C~N::prepare(archetype)?;
                });

                Some(($(
                    lower!($name)
                ),+,))
            }

            unsafe fn read(ptrs: Self::PtrTuple, i: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::read(ptrs.N, i);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        // Implement the immutable query layout for the tuples
        impl<'a, $($name: RefQueryItem<'a>),+> RefQueryLayout<'a> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);

            fn is_valid() -> bool {
                ($(mask::<$name::Component>())&+) == Mask::zero()
            }

            fn access(m: Mask) -> Option<LayoutAccess> {
                Some(($($name::access(m)?)|+))
            }

            fn prepare(archetype: &Archetype) -> Option<Self::PtrTuple> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    let c~N = C~N::prepare(archetype)?;
                });

                Some(($(
                    lower!($name)
                ),+,))
            }

            unsafe fn read(ptrs: Self::PtrTuple, i: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::read(ptrs.N, i);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }
    };
}

/*
tuple_impls! { C0 C1, 2 }
tuple_impls! { C0 C1 C2, 3 }
tuple_impls! { C0 C1 C2 C3, 4 }
tuple_impls! { C0 C1 C2 C3 C4, 5 }
tuple_impls! { C0 C1 C2 C3 C4 C5, 6 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6, 7 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7, 8 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }
*/
*/