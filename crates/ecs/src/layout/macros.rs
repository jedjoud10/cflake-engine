use crate::{
    mask, Archetype, Bundle, Component, LayoutAccess, Mask, MaskHashMap, QueryItemMut,
    QueryItemRef, QueryLayoutMut, QueryLayoutRef, StateColumn, StateFlags, UntypedVec,
};
use casey::lower;
use paste::paste;
use seq_macro::seq;

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {
        impl<$($name: Component),+> Bundle for ($($name,)+) {
            fn reduce(mut lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
                let masks = [$(mask::<$name>()),+];
                masks[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            fn extend_from_iter<'a>(
                archetype: &'a mut Archetype,
                iter: impl IntoIterator<Item = Self>
            ) -> Option<usize> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    #[allow(non_snake_case)]
                    let (components_C~N, states_C~N) = archetype.column_mut::<C~N>()?;
                    #[allow(non_snake_case)]
                    let components_ptr_C~N = components_C~N as *mut Vec::<C~N>;
                    #[allow(non_snake_case)]
                    let states_ptr_C~N = states_C~N as *mut StateColumn;
                    #[allow(non_snake_case)]
                    let components_C~N = unsafe { &mut *components_ptr_C~N };
                    #[allow(non_snake_case)]
                    let states_C~N = unsafe { &mut *states_ptr_C~N };
                });

                let mut storages = ($((
                    paste! { [<components_ $name>] }, paste! { [<states_ $name>] }
                )),+,);

                let mut additional = 0;

                seq!(N in 0..$max {
                    let column~N = &mut storages.N;
                });

                for bundle in iter.into_iter() {
                    seq!(N in 0..$max {
                        column~N.0.push(bundle.N);
                    });
                    additional += 1;
                }

                seq!(N in 0..$max {
                    column~N.1.extend_with_flags(additional, StateFlags {
                        added: true,
                        modified: true,
                    });
                    assert_eq!(column~N.0.len(), column~N.1.len());
                });

                Some(additional)
            }

            fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>> {
                let mut map = MaskHashMap::<Box<dyn UntypedVec>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::new(Vec::<$name>::new()))
                ),+);
                map
            }
        }

        impl<$($name: QueryItemRef, )+> QueryLayoutRef for ($($name,)+) {
            type OwnedTuple = ($($name::Owned,)+);
            type PtrTuple = ($($name::Ptr,)+);
            type SliceTuple<'s> = ($($name::Slice<'s>,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            unsafe fn ptrs_from_archetype_unchecked(archetype: &Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s> {
                seq!(N in 0..$max {
                    let c~N = C~N::from_raw_parts(ptrs.N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn read_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemRef>::read_unchecked(ptrs.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        impl<$($name: QueryItemMut, )+> QueryLayoutMut for ($($name,)+) {
            type OwnedTuple = ($($name::Owned,)+);
            type PtrTuple = ($($name::Ptr,)+);
            type SliceTuple<'s> = ($($name::Slice<'s>,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            unsafe fn ptrs_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_mut_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s> {
                seq!(N in 0..$max {
                    let c~N = C~N::from_raw_parts(ptrs.N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn read_mut_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemMut>::read_mut_unchecked(ptrs.N, index);
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

#[cfg(feature = "extended-tuples")]
mod extend {
    use crate::{
        mask, Archetype, Component, ComponentColumn, LayoutAccess, Mask, MaskHashMap, OwnedBundle,
        QueryItemMut, QueryItemRef, QueryLayoutMut, QueryLayoutRef,
    };
    use casey::lower;
    use seq_macro::seq;
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10, 11 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11, 12 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12, 13 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13, 14 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14, 15 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15, 16 }
}

#[cfg(feature = "extended-tuples")]
use extend::*;
