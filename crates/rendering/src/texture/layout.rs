use crate::object::Shared;
use std::mem::size_of;

// Defines how texels must be stored within textures
pub trait TexelLayout: 'static + Default {
    const INTERNAL_FORMAT: u32;
    const FORMAT: u32;
    const TYPE: u32;

    const CHANNELS: u32;
    const BYTES_PER_CHANNEL: u32;

    // Count the number of bytes that make each texel
    fn bytes() -> u32 {
        Self::BYTES_PER_CHANNEL * Self::CHANNELS
    }
}

// A range texel limiter that will hint the texture that the integer must be accessed as a floating point value, and that it must be in the 0-1 range
#[derive(Clone, Copy, Default)]
pub struct Ranged<T: Shared>(T);

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in the -1 - 1 range
#[derive(Clone, Copy, Default)]
pub struct Normalized<T: Shared>(T);

// Multiple channels
pub struct R<T: Shared>(T);
pub struct RG<T: Shared>(vek::Vec2<T>);
pub struct RGB<T: Shared>(vek::Vec3<T>);
pub struct RGBA<T: Shared>(vek::Vec4<T>);

// Unique depth and stencil channels
pub struct Depth<T: Shared>(T);
pub struct Stencil<T: Shared>(T);

impl<T: Default + Shared> Default for R<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default + Shared> Default for RG<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default + Shared> Default for RGB<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default + Shared> Default for RGBA<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default + Shared> Default for Depth<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default + Shared> Default for Stencil<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// Macro that will automatically implement the texel layout of all integers / floats for a specific channel layout type
macro_rules! impl_texel_layout {
    ($t:ident, $count:expr, $f: ident) => {
        paste::paste! {
            impl TexelLayout for $t<u32> {
                const FORMAT: u32 = gl::[<$f>];
                const INTERNAL_FORMAT: u32 = gl::[<$t 32UI>];
                const CHANNELS: u32 = $count;
                const TYPE: u32 = gl::UNSIGNED_INT;
                const BYTES_PER_CHANNEL: u32 = u32::BITS * 8;
            }

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

// Implement le funny for main channels
impl_texel_layout!(R, 1, RED);
impl_texel_layout!(RG, 2, RG);
impl_texel_layout!(RGB, 3, RGB);
impl_texel_layout!(RGBA, 4, RGBA);
