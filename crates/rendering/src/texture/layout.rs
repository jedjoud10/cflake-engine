use crate::buffer::GPUSendable;
use std::mem::size_of;

// Defines how texels must be stored within textures
pub trait TexelLayout {
    const GL_TYPE: u32;
    const CHANNELS: u32;
    const BYTES_PER_CHANNEL: u32;

    // Count the number of bytes that make each texel
    fn bytes() -> u32 {
        Self::BYTES_PER_CHANNEL * Self::CHANNELS
    }
}

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
#[derive(Clone, Copy)]
pub struct Ranged<T: GPUSendable>(T);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
#[derive(Clone, Copy)]
pub struct Normalized<T: GPUSendable>(T);

// Multiple channels
pub struct R<T: GPUSendable>(T);
pub struct RG<T: GPUSendable>(vek::Vec2<T>);
pub struct RGB<T: GPUSendable>(vek::Vec3<T>);
pub struct RGBA<T: GPUSendable>(vek::Vec4<T>);

// Macro that will automatically implement the texel layout of all integers / floats for a specific channel layout type
macro_rules! impl_texel_layout {
    ($c:expr, $t:ident, $count:expr) => {
        paste::paste! {
            impl TexelLayout for $t<u32> {
                const GL_TYPE: u32 = gl::[<$t 32UI>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u32::BITS * 8;
            }

            impl TexelLayout for $t<i32> {
                const GL_TYPE: u32 = gl::[<$t 32I>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i32::BITS * 8;
            }

            impl TexelLayout for $t<u16> {
                const GL_TYPE: u32 = gl::[<$t 16UI>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
            }

            impl TexelLayout for $t<i16> {
                const GL_TYPE: u32 = gl::[<$t 16I>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
            }

            impl TexelLayout for $t<u8> {
                const GL_TYPE: u32 = gl::[<$t 8UI>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
            }

            impl TexelLayout for $t<i8> {
                const GL_TYPE: u32 = gl::[<$t 8I>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
            }

            impl TexelLayout for $t<f32> {
                const GL_TYPE: u32 = gl::[<$t 32F>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = size_of::<f32>() as _;
            }

            impl TexelLayout for $t<Ranged<u16>> {
                const GL_TYPE: u32 = gl::[<$t 16>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u16::BITS * 8;
            }

            impl TexelLayout for $t<Normalized<i16>> {
                const GL_TYPE: u32 = gl::[<$t 16_SNORM>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i16::BITS * 8;
            }

            impl TexelLayout for $t<Ranged<u8>> {
                const GL_TYPE: u32 = gl::[<$t 8>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u8::BITS * 8;
            }

            impl TexelLayout for $t<Normalized<i8>> {
                const GL_TYPE: u32 = gl::[<$t 8_SNORM>];
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i8::BITS * 8;
            }
        }
    };
}

// Implement le funny
impl_texel_layout!(R, R, 1);
impl_texel_layout!(RG, RG, 2);
impl_texel_layout!(RGB, RGB, 3);
impl_texel_layout!(RGBA, RGBA, 4);