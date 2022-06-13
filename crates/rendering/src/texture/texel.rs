use crate::object::Shared;
use super::channels::{R, RG, RGB, RGBA};
use super::{Normalized, Ranged};
use std::mem::size_of;
use vek::{Vec2, Vec3, Vec4};

// This trait defines the layout for a single texel that will be stored within textures1
pub trait Texel: 'static {
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

    // Raw texel type that we store internally and that the user will interact with
    type Storage: Shared; 

    // Count the number of bytes that make each texel
    fn bytes() -> u32 {
        Self::BYTES_PER_CHANNEL * Self::CHANNELS
    }   
}


// Macro that will automatically implement the texel layout trait
macro_rules! impl_texel_layout {
    ($t:ident, $count:expr, $f: ident, $vec: ident) => {
        paste::paste! {
            impl Texel for $t<u32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_INT;
                const BYTES_PER_CHANNEL: u32 = u32::BITS * 8;
                type Storage = $vec<u32>;
            }

            impl Texel for $t<i32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::INT;
                const BYTES_PER_CHANNEL: u32 = i32::BITS * 8;
                type Storage = $vec<i32>;
            }

            impl Texel for $t<u16> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
                type Storage = $vec<u16>;
            }

            impl Texel for $t<i16> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::SHORT;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
                type Storage = $vec<i16>;                
            }

            impl Texel for $t<u8> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
                type Storage = $vec<u8>;
            }

            impl Texel for $t<i8> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8I>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::BYTE;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
                type Storage = $vec<i8>;
            }

            impl Texel for $t<f32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32F>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::FLOAT;
                const BYTES_PER_CHANNEL: u32 = size_of::<f32>() as _;
                type Storage = $vec<f32>;
            }

            impl Texel for $t<Ranged<u16>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
                type Storage = $vec<u16>;
            }

            impl Texel for $t<Normalized<i16>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 16_SNORM>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::SHORT;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
                type Storage = $vec<i16>;
            }

            impl Texel for $t<Ranged<u8>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
                type Storage = $vec<u8>;
            }

            impl Texel for $t<Normalized<i8>> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 8_SNORM>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::BYTE;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
                type Storage = $vec<i8>;
            }
        }
    };
}

// Need this for the macro to work
type Scalar<T> = T;

// Implement main texel layout trait
impl_texel_layout!(R, 1, RED, Scalar);
impl_texel_layout!(RG, 2, RG, Vec2);
impl_texel_layout!(RGB, 3, RGB, Vec3);
impl_texel_layout!(RGBA, 4, RGBA, Vec4);