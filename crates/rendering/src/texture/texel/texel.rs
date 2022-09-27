use super::channels::*;
use super::element::*;
use crate::context::Shared;
use std::mem::size_of;
use vek::Vec2;
use vek::Vec3;
use vek::Vec4;

// An untyped wrapper around texel types
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct UntypedTexel {
    pub internal_format: u32,
    pub format: u32,
    pub type_: u32,
    pub channels: u32,
    pub bytes_per_channel: u32,
    pub enum_format: TexelFormat,
}

// The "type" of texel layout we're dealing with
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TexelFormat {
    Color,
    Depth,
    Stencil,
    GammaCorrectedColor,
}

// This trait defines the layout for a single texel that will be stored within textures1
pub trait Texel: 'static + Sized {
    // Main OpenGL wrapper enums / values
    const INTERNAL_FORMAT: u32;
    const FORMAT: u32;
    const TYPE: u32;
    const CHANNELS: u32;
    const BYTES_PER_CHANNEL: u32;
    const ENUM_FORMAT: TexelFormat;

    // Storage is the blit type, like Vec3 or Scalar that contains Element
    type Storage: Shared;
    type Element: Shared;

    fn bytes() -> u32 {
        Self::BYTES_PER_CHANNEL * Self::CHANNELS
    }

    // Convert the texel type to an untypped wrapper
    fn untyped() -> UntypedTexel {
        UntypedTexel {
            internal_format: Self::INTERNAL_FORMAT,
            format: Self::FORMAT,
            type_: Self::TYPE,
            channels: Self::CHANNELS,
            bytes_per_channel: Self::BYTES_PER_CHANNEL,
            enum_format: Self::ENUM_FORMAT,
        }
    }
}

// Implement the color texel layout
macro_rules! impl_color_texel_layout {
    ($t:ident, $count:expr, $f: ident, $vec: ident) => {
        paste::paste! {
            impl Texel for $t<u32> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 32UI>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::UNSIGNED_INT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u32::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<u32>;
                type Element = u32;
            }

            impl Texel for $t<i32> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 32I>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::INT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i32::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<i32>;
                type Element = i32;
            }

            impl Texel for $t<u16> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 16UI>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u16::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<u16>;
                type Element = u16;
            }

            impl Texel for $t<i16> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 16I>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::SHORT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i16::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<i16>;
                type Element = i16;
            }

            impl Texel for $t<u8> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 8UI>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u8::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<u8>;
                type Element = u8;
            }

            impl Texel for $t<i8> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 8I>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::BYTE;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i8::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<i8>;
                type Element = i8;
            }

            impl Texel for $t<f32> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 32F>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::FLOAT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = size_of::<f32>() as _;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<f32>;
                type Element = f32;
            }

            impl Texel for $t<Ranged<u16>> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 16>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::UNSIGNED_SHORT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u16::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<u16>;
                type Element = u16;
            }

            impl Texel for $t<Normalized<i16>> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 16_SNORM>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::SHORT;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i16::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<i16>;
                type Element = i16;
            }

            impl Texel for $t<Ranged<u8>> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 8>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::UNSIGNED_BYTE;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = u8::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<u8>;
                type Element = u8;
            }

            impl Texel for $t<Normalized<i8>> {
                const INTERNAL_FORMAT: u32 = gl::[<$t 8_SNORM>];
                const FORMAT: u32 = gl::[<$f>];
                const TYPE: u32 = gl::BYTE;
                const CHANNELS: u32 = $count;
                const BYTES_PER_CHANNEL: u32 = i8::BITS / 8;
                const ENUM_FORMAT: TexelFormat = TexelFormat::Color;
                type Storage = $vec<i8>;
                type Element = i8;
            }
        }
    };
}

// Implement the depth texel layout
macro_rules! impl_depth_texel_layout {
    () => {
        impl Texel for Depth<Ranged<u16>> {
            const INTERNAL_FORMAT: u32 = gl::DEPTH_COMPONENT16;
            const FORMAT: u32 = gl::DEPTH_COMPONENT;
            const TYPE: u32 = gl::UNSIGNED_SHORT;
            const CHANNELS: u32 = 1;
            const BYTES_PER_CHANNEL: u32 = u16::BITS / 8;
            const ENUM_FORMAT: TexelFormat = TexelFormat::Depth;
            type Storage = Scalar<u16>;
            type Element = u16;
        }

        impl Texel for Depth<Ranged<u32>> {
            const INTERNAL_FORMAT: u32 = gl::DEPTH_COMPONENT32;
            const FORMAT: u32 = gl::DEPTH_COMPONENT;
            const TYPE: u32 = gl::UNSIGNED_INT;
            const CHANNELS: u32 = 1;
            const BYTES_PER_CHANNEL: u32 = u32::BITS / 8;
            const ENUM_FORMAT: TexelFormat = TexelFormat::Depth;
            type Storage = Scalar<u32>;
            type Element = u32;
        }

        impl Texel for Depth<f32> {
            const INTERNAL_FORMAT: u32 = gl::DEPTH_COMPONENT32F;
            const FORMAT: u32 = gl::DEPTH_COMPONENT;
            const TYPE: u32 = gl::FLOAT;
            const CHANNELS: u32 = 1;
            const BYTES_PER_CHANNEL: u32 = size_of::<f32>() as _;
            const ENUM_FORMAT: TexelFormat = TexelFormat::Depth;
            type Storage = Scalar<f32>;
            type Element = f32;
        }
    };
}

// Implement the stencil texel layout
macro_rules! impl_stencil_texel_layout {
    () => {
        impl Texel for Stencil<u8> {
            const INTERNAL_FORMAT: u32 = gl::STENCIL_INDEX8;
            const FORMAT: u32 = gl::STENCIL_INDEX;
            const TYPE: u32 = gl::UNSIGNED_BYTE;
            const CHANNELS: u32 = 1;
            const BYTES_PER_CHANNEL: u32 = u8::BITS / 8;
            const ENUM_FORMAT: TexelFormat = TexelFormat::Stencil;
            type Storage = Scalar<u8>;
            type Element = u8;
        }
    };
}

// Implement the gamma corrected texel layout
macro_rules! impl_gamma_corrected_texel_layout {
    () => {
        impl Texel for SRGB<Ranged<u8>> {
            const INTERNAL_FORMAT: u32 = gl::SRGB8;
            const FORMAT: u32 = gl::RGB;
            const TYPE: u32 = gl::UNSIGNED_BYTE;
            const CHANNELS: u32 = 3;
            const BYTES_PER_CHANNEL: u32 = u8::BITS / 8;
            const ENUM_FORMAT: TexelFormat = TexelFormat::GammaCorrectedColor;
            type Storage = Vec3<u8>;
            type Element = u8;
        }

        impl Texel for SRGBA<Ranged<u8>> {
            const FORMAT: u32 = gl::RGBA;
            const INTERNAL_FORMAT: u32 = gl::SRGB8_ALPHA8;
            const TYPE: u32 = gl::UNSIGNED_BYTE;
            const CHANNELS: u32 = 4;
            const BYTES_PER_CHANNEL: u32 = u8::BITS / 8;
            const ENUM_FORMAT: TexelFormat = TexelFormat::GammaCorrectedColor;
            type Storage = Vec4<u8>;
            type Element = u8;
        }
    };
}

// Need this for the macro to work
type Scalar<T> = T;

// Implement main texel layout trait
impl_color_texel_layout!(R, 1, RED, Scalar);
impl_color_texel_layout!(RG, 2, RG, Vec2);
impl_color_texel_layout!(RGB, 3, RGB, Vec3);
impl_color_texel_layout!(RGBA, 4, RGBA, Vec4);

// Others
impl_depth_texel_layout!();
impl_stencil_texel_layout!();
impl_gamma_corrected_texel_layout!();
