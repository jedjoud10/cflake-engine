use crate::{
    mask, Archetype, Component, ComponentColumn, LayoutAccess, Mask,
    MaskHashMap, OwnedBundle, QueryItemMut, QueryItemRef,
    QueryLayoutMut, QueryLayoutRef,
};
use casey::lower;
use seq_macro::seq;

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {
        impl<'a, $($name: Component),+> OwnedBundle<'a> for ($($name,)+) {
            type Storages = ($(&'a mut Vec<$name>),+);

            fn reduce(mut lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
                let masks = [$(mask::<$name>()),+];
                masks[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    let table = archetype.components_mut::<C~N>()?;
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

            fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>> {
                let mut map = MaskHashMap::<Box<dyn ComponentColumn>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }

            fn try_swap_remove(tables: &mut MaskHashMap<Box<dyn ComponentColumn>>, index: usize) -> Option<Self> {
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

        impl<'i, $($name: QueryItemRef<'i> + 'i, )+> QueryLayoutRef<'i> for ($($name,)+) {
            type OwnedTuple = ($($name::Owned,)+);
            type PtrTuple = ($($name::Ptr,)+);
            type SliceTuple = ($($name::Slice,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().filter(|l| *l != LayoutAccess::none()).reduce(|a, b| lambda(a, b)).unwrap()
            }

            unsafe fn ptrs_from_archetype_unchecked(archetype: &Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn from_raw_parts(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::from_raw_parts(ptrs.N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn read_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemRef<'i>>::read_unchecked(ptrs.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        impl<'i, $($name: QueryItemMut<'i> + 'i, )+> QueryLayoutMut<'i> for ($($name,)+) {
            type OwnedTuple = ($($name::Owned,)+);
            type PtrTuple = ($($name::Ptr,)+);
            type SliceTuple = ($($name::Slice,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().filter(|l| *l != LayoutAccess::none()).reduce(|a, b| lambda(a, b)).unwrap()
            }

            unsafe fn ptrs_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_mut_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn from_raw_parts(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::from_raw_parts(ptrs.N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn read_mut_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemMut<'i>>::read_mut_unchecked(ptrs.N, index);
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

#[cfg(feature = "extended-tuples")]
mod extend {
    use crate::{
        mask, name, Archetype, Component, ComponentColumn,
        LayoutAccess, Mask, MaskHashMap, OwnedBundle, QueryItemMut,
        QueryItemRef, QueryLayoutMut, QueryLayoutRef,
    };
    use casey::lower;
    use seq_macro::seq;
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6, 7 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7, 8 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10, 11 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11, 12 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12, 13 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13, 14 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14, 15 }
}

#[cfg(feature = "extended-tuples")]
use extend::*;
