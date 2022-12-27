use std::marker::PhantomData;
use bytemuck::Pod;


// Base numbers that are used to store the inner raw values of texture texels
pub trait Base: Pod + Clone + Send + Sync {
    const TYPE: BaseType; 
}

impl Base for i8 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u8 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i16 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u16 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i32 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u32 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for i64 { const TYPE: BaseType = BaseType::SignedInt; }
impl Base for u64 { const TYPE: BaseType = BaseType::UnsignedInt; }
impl Base for f32 { const TYPE: BaseType = BaseType::Float; }

// Untyped representation needed for texel
pub enum BaseType {
    UnsignedInt,
    SignedInt,
    Float
}

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait AnyElement {}
impl<T: Base> AnyElement for T {}

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in
//  the -1 - 1 range if it's a signed integer and the 0 - 1 range if it's an unsigned integer
pub struct Normalized<T: Base>(PhantomData<T>);
impl<T: Base> AnyElement for Normalized<T> {}