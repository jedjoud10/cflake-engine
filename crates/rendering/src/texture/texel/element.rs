use std::marker::PhantomData;

// Base numbers that are used to store the inner raw values of texture texels
pub trait Base {}
impl Base for i8 {}
impl Base for u8 {}
impl Base for i16 {}
impl Base for u16 {}
impl Base for i32 {}
impl Base for u32 {}
impl Base for f32 {}

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait Element {}
impl<T: Base> Element for T {}

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
pub struct Ranged<T: Base>(PhantomData<T>);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
pub struct Normalized<T: Base>(PhantomData<T>);

impl<T: Base> Element for Ranged<T> {}
impl<T: Base> Element for Normalized<T> {}