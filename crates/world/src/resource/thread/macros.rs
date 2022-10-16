use super::*;
use arrayvec::ArrayVec;
use casey::lower;
use seq_macro::seq;
use std::slice::from_raw_parts;

macro_rules! impl_tuple {
    ($( $name:ident )+, $max:tt) => {
        impl<'s: 'i, 'i, $($name: RefSlice<'s, 'i> + 's),+> RefSliceTuple<'s, 'i> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);
            type ItemRefTuple = ($($name::ItemRef),+);

            fn as_ptrs(&self) -> Self::PtrTuple {
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

                (vec.is_empty()).then(|| vec).map(|vec| vec.windows(2).all(|a| a[0] == a[1]).then(|| vec[0])).flatten()
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

            unsafe fn get_unchecked(&self, index: usize) -> Self::ItemRefTuple {
                seq!(N in 0..$max {
                    let c~N: C~N::ItemRef = C~N::get_unchecked(&self.N, index);
                });

                ($(
                    lower!($name)
                ),+,)
            }
        }

        impl<'s: 'i, 'i, $($name: MutSlice<'s, 'i> + 's),+> MutSliceTuple<'s, 'i> for ($($name,)+) {
            type PtrTuple = ($($name::Ptr),+);
            type ItemRefTuple = ($($name::ItemRef),+);

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

                (vec.is_empty()).then(|| vec).map(|vec| vec.windows(2).all(|a| a[0] == a[1]).then(|| vec[0])).flatten()
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

            unsafe fn get_unchecked<'s2: 'i>(&'s2 mut self, index: usize) -> Self::ItemRefTuple {
                seq!(N in 0..$max {
                    let c~N: C~N::ItemRef = C~N::get_unchecked(&mut self.N, index);
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
