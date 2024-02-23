#![allow(non_snake_case)]

use super::{Bundle, LayoutAccess, QueryItemMut, QueryItemRef, QueryLayoutMut, QueryLayoutRef};
use crate::archetype::{Archetype, StateColumn, StateFlags};
use crate::mask::{Mask, MaskHashMap};
use crate::registry::{mask, Component};
use crate::vec::UntypedVec;
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
                moved: bool,
                iter: impl IntoIterator<Item = Self>
            ) -> Option<usize> {
                assert!(Self::is_valid());
                seq!(N in 0..$max {
                    let (components_C~N, states_C~N) = archetype.column_mut::<C~N>()?;
                    let components_ptr_C~N = components_C~N as *mut Vec::<C~N>;
                    let states_ptr_C~N = states_C~N as *mut StateColumn;
                    let components_C~N = unsafe { &mut *components_ptr_C~N };
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
                        added: !moved,
                        modified: !moved,
                    });
                });

                Some(additional)
            }

            fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>> {
                let mut map = MaskHashMap::<Box<dyn UntypedVec>>::default();
                ($(
                    map.insert(mask::<$name>(), Box::<Vec::<$name>>::default())
                ),+);
                map
            }
        }

        impl<'s, $($name: QueryItemRef<'s>, )+> QueryLayoutRef<'s> for ($($name,)+) {
            type SliceTuple = ($($name::Slice,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            #[inline]
            fn from_archetype(archetype: &'s Archetype) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::from_archetype(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            #[inline]
            fn read(slice: Self::SliceTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemRef>::read(slice.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        impl<'s, $($name: QueryItemMut<'s>, )+> QueryLayoutMut<'s> for ($($name,)+) {
            type SliceTuple = ($($name::Slice,)+);

            fn reduce(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
            }

            #[inline]
            fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::SliceTuple {
                let ptr = archetype as *mut Archetype;
                
                seq!(N in 0..$max {
                    let archetype~N = unsafe { &mut *ptr };
                    let c~N = C~N::from_mut_archetype(archetype~N);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            #[inline]
            fn read_mut(slice: Self::SliceTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = <C~N as QueryItemMut>::read_mut(slice.N, index);
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
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10, 11 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11, 12 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12, 13 }
tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13, 14 }


/*
#[cfg(feature = "extended-tuples")]
mod extend {
    use crate::{
        mask, Archetype, Component, ComponentColumn, LayoutAccess, Mask, MaskHashMap, OwnedBundle,
        QueryItemMut, QueryItemRef, QueryLayoutMut, QueryLayoutRef,
    };
    use casey::lower;
    use seq_macro::seq;
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14, 15 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15, 16 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15 C16, 17 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15 C16 C17, 18 }
    tuple_impls! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15 C16 C17 C18, 19 }
}

#[cfg(feature = "extended-tuples")]
use extend::*;
*/