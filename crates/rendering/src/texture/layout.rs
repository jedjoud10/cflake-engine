use vek::{Vec3, Vec2, Vec4};

use crate::object::Shared;
use std::{mem::size_of, marker::PhantomData};

// This trait defines the layout for a single texel that will be stored within textures1
pub trait TexelLayout: 'static {
    // Corresponds to the OpenGL internal format parameter
    const INTERNAL_FORMAT: u32;

    // Corresponds to the OpenGl format parameter
    const FORMAT: u32;

    // Corresponds to the OpenGL data type parameter
    const TYPE: u32;

    // The number of channels that we have stored within the texel
    const CHANNELS: u32;

    // The number of bytes per channel
    const BYTES_PER_CHANNEL: u32;

    // The data that is actually stored within the texels
    type Raw;

    // The data that the user will interact with (input, output)
    type User;

    // Count the number of bytes that make each texel
    fn bytes() -> u32 {
        Self::BYTES_PER_CHANNEL * Self::CHANNELS
    }

    // Convert the raw texel data to it's user repr
    fn texel_from_raw_repr(raw: Self::Raw) -> Self::User;

    // Convert the user texel data to it's raw repr
    fn texel_into_raw_repr(user: Self::User) -> Self::Raw;
}

// This trait defines one single element that is stored whithin the texel's channels
trait Elem {}

// Implementation of elem for default types
impl Elem for i8 {}
impl Elem for u8 {}
impl Elem for i16 {}
impl Elem for u16 {}
impl Elem for i32 {}
impl Elem for u32 {}
impl Elem for f32 {}

// Limiters convert the full binary numbers into some sort of fixed point representation
// For our implementation, limiters only convert to f32 
trait Limiter: Elem {
}

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
pub struct Ranged<T: Elem>(PhantomData<T>);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
pub struct Normalized<T: Elem>(PhantomData<T>);

// Implementations of elem for limiters
impl Elem for Ranged<u8> {}
impl Elem for Ranged<u16> {}
impl Elem for Ranged<u32> {}
impl Elem for Normalized<i8> {}
impl Elem for Normalized<i16> {}
impl Elem for Normalized<i32> {}

// Implementations of limiter for limiters
impl Limiter for Ranged<u8> {}
impl Limiter for Ranged<u16> {}
impl Limiter for Ranged<u32> {}
impl Limiter for Normalized<i8> {}
impl Limiter for Normalized<i16> {}
impl Limiter for Normalized<i32> {}

// The channels that represent the texels
pub struct R<T: Elem>(PhantomData<T>);
pub struct RG<T: Elem>(PhantomData<Vec2<T>>);
pub struct RGB<T: Elem>(PhantomData<Vec3<T>>);
pub struct RGBA<T: Elem>(PhantomData<Vec4<T>>);

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: Elem>(PhantomData<T>);
pub struct Stencil<T: Elem>(PhantomData<T>);

impl TexelLayout for RG<Normalized<i16>> {
    const FORMAT: u32 = gl::RG;
    const INTERNAL_FORMAT: u32 = gl::RG16_SNORM;
    const CHANNELS: u32 = 2;
    const TYPE: u32 = gl::SHORT;
    const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;

    type Raw = vek::Vec2<i16>;

    type User = vek::Vec2<f32>;

    fn texel_from_raw_repr(raw: Self::Raw) -> Self::User {
        raw.as_::<f32>().map(|x| x / i16::MAX as f32)
    }

    fn texel_into_raw_repr(user: Self::User) -> Self::Raw {
        user.map(|x| x * i16::MAX).as_::<i16>()
    }
}

// Macro that will automatically implement the texel layout trait
macro_rules! impl_texel_layout {
    ($t:ident, $count:expr, $f: ident) => {
        paste::paste! {
            /*
            impl TexelLayout for $t<u32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_INT;
                const BYTES_PER_CHANNEL: u32 = u32::BITS * 8;
            }
            */

            impl TexelLayout for $t<i32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::INT;
                const BYTES_PER_CHANNEL: u32 = i32::BITS * 8;
            }

            impl TexelLayout for $t<u16> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
                
            }

            impl TexelLayout for $t<i16> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::SHORT;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
                
            }

            impl TexelLayout for $t<u8> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
                
            }

            impl TexelLayout for $t<i8> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::BYTE;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
                
            }

            impl TexelLayout for $t<f32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32F>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::FLOAT;
                const BYTES_PER_CHANNEL: u32 = size_of::<f32>() as _;
                
            }

            impl TexelLayout for $t<Ranged<u16>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
                
            }

            impl TexelLayout for $t<Normalized<i16>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16_SNORM>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::SHORT;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
                
            }

            impl TexelLayout for $t<Ranged<u8>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
                
            }

            impl TexelLayout for $t<Normalized<i8>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8_SNORM>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::BYTE;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
                
            }
        }
    };
}

// Implement main texel layout trait
impl_texel_layout!(R, 1, RED);
impl_texel_layout!(RG, 2, RG);
impl_texel_layout!(RGB, 3, RGB);
impl_texel_layout!(RGBA, 4, RGBA);
/*
impl TexelLayout for R<u32> {
    const FORMAT: u32 = gl::RED;
    const INTERNAL_FORMAT: u32 = gl::R32UI;
    const CHANNELS: u32 = 1;
    const TYPE: u32 = gl::UNSIGNED_INT;
    const BYTES_PER_CHANNEL: u32 = u32::BITS * 8;
}
*/