use super::*;
use arrayvec::ArrayVec;
use casey::lower;
use seq_macro::seq;

macro_rules! impl_tuple {
    ($( $name:ident )+, $max:tt) => {
        impl<'i, $($name: Slice<'i>),+> SliceTuple<'i> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);
            type OwnedTuple = ($($name::OwnedItem),+);
            type ItemTuple = ($($name::Item),+);

            fn as_ptrs(&mut self) -> Self::PtrTuple {
                seq!(N in 0..$max {
                    let c~N: C~N::Ptr = (self.N).as_ptr();
                });

                ($(
                    lower!($name)
                ),+,)
            }

            fn slice_tuple_len(&self) -> Option<usize> {
                let mut vec: ArrayVec<usize, $max> = ArrayVec::new();
                seq!(N in 0..$max {
                    if let Some(len) = self.N.len() {
                        vec.push(len);
                    }
                });

                (!vec.is_empty()).then(|| vec).map(|vec| {
                    let first = vec[0];
                    let others = vec.iter().all(|l| *l == first);
                    others.then_some(first)
                }).unwrap_or_default()
            }

            unsafe fn from_ptrs(ptrs: &Self::PtrTuple, length: usize, offset: usize) -> Self {
                seq!(N in 0..$max {
                    let ptr~N: C~N::Ptr = C~N::offset_ptr(ptrs.N, offset);
                    let c~N: C~N = C~N::from_raw_parts(ptr~N, length);
                });

                ($(
                    lower!($name)
                ),+,)
            }

            unsafe fn get_unchecked<'a: 'i>(&'a mut self, index: usize) -> Self::ItemTuple {
                seq!(N in 0..$max {
                    let c~N: C~N::Item = C~N::get_unchecked(&mut self.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }
    };
}

impl_tuple! { C0 C1, 2 }
impl_tuple! { C0 C1 C2, 3 }
impl_tuple! { C0 C1 C2 C3, 4 }
impl_tuple! { C0 C1 C2 C3 C4, 5 }
impl_tuple! { C0 C1 C2 C3 C4 C5, 6 }
impl_tuple! { C0 C1 C2 C3 C4 C5 C6, 7 }
impl_tuple! { C0 C1 C2 C3 C4 C5 C6 C7, 8 }
impl_tuple! { C0 C1 C2 C3 C4 C5 C6 C7 C8, 9 }
impl_tuple! { C0 C1 C2 C3 C4 C5 C6 C7 C8 C9, 10 }
