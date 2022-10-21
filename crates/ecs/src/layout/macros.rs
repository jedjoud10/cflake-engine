use crate::{QueryLayout, QueryItem, QueryValidityError, LayoutAccess, Archetype, OwnedBundle};
use casey::lower;
use seq_macro::seq;

macro_rules! tuple_impls {
    ( $( $name:ident )+, $max:tt ) => {           
        impl<'s: 'i, 'i, $($name: QueryItem<'s, 'i>, )+> QueryLayout<'s, 'i> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr,)+);
            type SliceTuple = ($($name::Slice,)+);
        
            fn items() -> usize {
                $max
            }
        
            fn name(index: usize) -> &'static str {
                let names = [$($name::name()),+];
                names[index]
            }
        
            fn fold(mut lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                let layouts = [$($name::access()),+];
                let first = layouts[0];
                layouts[1..].into_iter().fold(first, |a, b| lambda(a, *b))
            }

            fn mutable() -> Option<usize> {
                let mutable = [$($name::MUTABLE),+];
                mutable.into_iter().position(|v| v)
            }

            unsafe fn ptrs_from_archetype_unchecked(archetype: &Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }
            
            unsafe fn ptrs_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::ptr_from_mut_archetype_unchecked(archetype);
                });

                ($(
                    lower!($name)
                ),+,)
            }            
        
            unsafe fn from_raw_parts(tuple: Self::PtrTuple, length: usize) -> Self::SliceTuple {
                seq!(N in 0..$max {
                    let c~N = C~N::from_raw_parts(tuple.N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        
            unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self {
                seq!(N in 0..$max {
                    let c~N = C~N::get_unchecked(slice.N, index);
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