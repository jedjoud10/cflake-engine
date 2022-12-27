use std::marker::PhantomData;
use crate::{Base, BaseType};

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait AnyElement: 'static {
    const TYPE: BaseType; 
    const NORMALIZED: bool;
}
impl<T: Base> AnyElement for T {
    const TYPE: BaseType = T::TYPE;
    const NORMALIZED: bool = false;
}

// This trait represents bases that can be normalized
pub trait Normalizable: Base {}
impl Normalizable for i8 {}
impl Normalizable for u8 {}
impl Normalizable for i16 {}
impl Normalizable for u16 {}

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in
//  the -1 - 1 range if it's a signed integer and the 0 - 1 range if it's an unsigned integer
pub struct Normalized<T: Base + Normalizable>(T);
impl<T: Base + Normalizable> AnyElement for Normalized<T> {
    const TYPE: BaseType = T::TYPE;
    const NORMALIZED: bool = true;
}