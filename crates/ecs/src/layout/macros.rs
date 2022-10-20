use crate::{QueryLayout, QueryItem, QueryValidityError};

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
                
                
                if index == 0 {
                    I::name()
                } else {
                    panic!()
                }
            }
        
            fn fold(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
                [I::access()].into_iter().fold(LayoutAccess::none(), lambda)
            }
        
            fn ptrs_from_archetype(archetype: &Archetype) -> Self::PtrTuple {
                I::ptr_from_archetype(archetype)
            }
        
            fn ptrs_from_mut_archetype(archetype: &mut Archetype) -> Self::PtrTuple {
                I::ptr_from_mut_archetype(archetype)
            }
        
            unsafe fn from_raw_parts(tuple: Self::PtrTuple, length: usize) -> Self::SliceTuple {
                <I as QueryItem<'s, 'i>>::from_raw_parts(tuple, length)
            }
        
            unsafe fn get_unchecked(slice: Self::SliceTuple, index: usize) -> Self {
                <I as QueryItem<'s, 'i>>::get_unchecked(slice, index)
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