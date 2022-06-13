use std::marker::PhantomData;
use super::Element;

// Limiters convert the full binary numbers into some sort of fixed point representation
// For our implementation, limiters only convert to f32 
pub trait Limiter: Element {
    type Inner;

    // This checks if a float can be convert into the inner type successfully
    fn in_range(val: f32) -> bool;
    
    // Convert the inner stored type to the limited and mapped f32
    fn inner_into_f32(inner: Self::Inner) -> f32;

    // Convert a random f32 given by the user to the inner type
    // This assumes that the given float is in the specified range
    // This will overflow if the value is not in the proper range
    fn inner_from_f32(val: f32) -> Self::Inner;
}

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
pub struct Ranged<T: Element>(PhantomData<T>);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
pub struct Normalized<T: Element>(PhantomData<T>);


macro_rules! impl_limiter {
    ($unsigned:ident, $signed:ident) => {
        paste::paste! {
            impl Limiter for Ranged<$unsigned> {
                type Inner = $unsigned;

                fn in_range(val: f32) -> bool {
                    val >= 0.0 && val <= 1.0
                }
                
                fn inner_into_f32(inner: Self::Inner) -> f32 {
                    inner as f32 / Self::Inner::MAX as f32
                }
                
                fn inner_from_f32(val: f32) -> Self::Inner {
                    (val * Self::Inner::MAX as f32) as Self::Inner
                }
            }

            impl Limiter for Normalized<$signed> {
                type Inner = $signed;

                fn in_range(val: f32) -> bool {
                    val >= -1.0 && val <= 1.0
                }
                
                fn inner_into_f32(inner: Self::Inner) -> f32 {
                    inner as f32 / Self::Inner::MAX as f32
                }
                
                fn inner_from_f32(val: f32) -> Self::Inner {
                    (val * Self::Inner::MAX as f32) as Self::Inner
                }
            }
        }
    };
}

impl_limiter!(u8, i8);
impl_limiter!(u16, i16);

impl<T: Element> Element for Ranged<T> {}
impl<T: Element> Element for Normalized<T> {}