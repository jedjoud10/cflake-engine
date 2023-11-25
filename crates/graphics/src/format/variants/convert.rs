use crate::format::{
    AnyElement, Depth, DepthElement, Normalized, Stencil, StencilElement, Texel, BGRA, R, RG, RGBA,
    SBC4, SBC5, SBGRA, SRGBA, UBC1, UBC2, UBC3, UBC4, UBC5, UBC7,
};
use half::f16;

// This trait is used to convert between texel storage types to intermediate types
pub trait Conversion: Texel {
    // The corresponding intermediate type that should contain the data
    // This is quite different than Texel::Storage because this type can be shared by multiple texels at the same time
    // For color texels, this would be Vec4<f32>, since color texels should be represented by RGBA
    // For depth and stencil, this would be f32 and u32 respectively
    type Target;

    // Convert an intermediate storage type repr to a texel
    // Must return None if the conversion fails
    // Extra channels must be zeroed out
    fn try_from_target(target: Self::Target) -> Option<Self::Storage>;

    // Converts the color texel to the intermediate storage type
    // Extra channels must be zeroed out
    // Converting into the target type must always be successful, because the target type should be able to represent all of it's variants
    fn into_target(texel: Self::Storage) -> Self::Target;
}

fn map<T: Texel>(rgba: vek::Vec4<f32>, map: impl Fn(f32) -> T::Base) -> vek::Vec4<T::Base> {
    rgba.map(map)
}

// Do NOT FUCKING TOUCH THIS
macro_rules! internal_impl_color_texel {
    ($vec:ident, $elem:ty, $storagevec:ident, $conv:expr, $min:expr, $max:expr, $fromf32:expr, $tof32:expr) => {
        impl Conversion for $vec<$elem> {
            type Target = vek::Vec4<f32>;

            fn try_from_target(rgba: vek::Vec4<f32>) -> Option<Self::Storage> {
                if rgba.reduce_partial_max() > $max || rgba.reduce_partial_min() < $min {
                    return None;
                }

                let converted = map::<Self>(rgba, $fromf32);
                let killme = $conv;
                let storage = killme(converted);
                Some(storage)
            }

            fn into_target(texel: Self::Storage) -> vek::Vec4<f32> {
                let input = vek::Vec4::<Self::Base>::from(texel);
                let mapped = input.map($tof32);
                mapped
            }
        }
    };
}

// I deserve to be in the deepest layers of hell
macro_rules! impl_color_texels {
    ($vec:ident, $storagevec:ident, $conv:expr) => {
        internal_impl_color_texel!(
            $vec,
            u8,
            $storagevec,
            $conv,
            u8::MIN as f32,
            u8::MAX as f32,
            |f| f as u8,
            |v| v as f32
        );
        internal_impl_color_texel!(
            $vec,
            i8,
            $storagevec,
            $conv,
            i8::MIN as f32,
            i8::MAX as f32,
            |f| f as i8,
            |v| v as f32
        );
        internal_impl_color_texel!(
            $vec,
            Normalized<u8>,
            $storagevec,
            $conv,
            0.0,
            1.0,
            |f| (f * u8::MAX as f32) as u8,
            |v| v as f32 / u8::MAX as f32
        );
        internal_impl_color_texel!(
            $vec,
            Normalized<i8>,
            $storagevec,
            $conv,
            -1.0,
            1.0,
            |f| (f * i8::MAX as f32) as i8,
            |v| v as f32 / i8::MAX as f32
        );

        internal_impl_color_texel!(
            $vec,
            u16,
            $storagevec,
            $conv,
            u16::MIN as f32,
            u16::MAX as f32,
            |f| f as u16,
            |v| v as f32
        );
        internal_impl_color_texel!(
            $vec,
            i16,
            $storagevec,
            $conv,
            i16::MIN as f32,
            i16::MAX as f32,
            |f| f as i16,
            |v| v as f32
        );
        internal_impl_color_texel!(
            $vec,
            Normalized<u16>,
            $storagevec,
            $conv,
            0.0,
            1.0,
            |f| (f * u16::MAX as f32) as u16,
            |v| v as f32 / u16::MAX as f32
        );
        internal_impl_color_texel!(
            $vec,
            Normalized<i16>,
            $storagevec,
            $conv,
            -1.0,
            1.0,
            |f| (f * i16::MAX as f32) as i16,
            |v| v as f32 / i16::MAX as f32
        );

        internal_impl_color_texel!(
            $vec,
            u32,
            $storagevec,
            $conv,
            u32::MIN as f32,
            u32::MAX as f32,
            |f| f as u32,
            |v| v as f32
        );
        internal_impl_color_texel!(
            $vec,
            i32,
            $storagevec,
            $conv,
            i32::MIN as f32,
            i32::MAX as f32,
            |f| f as i32,
            |v| v as f32
        );

        internal_impl_color_texel!(
            $vec,
            f16,
            $storagevec,
            $conv,
            f32::from(f16::MIN),
            f32::from(f16::MAX),
            |f| f16::from_f32(f),
            |v| f32::from(v)
        );
        internal_impl_color_texel!(
            $vec,
            f32,
            $storagevec,
            $conv,
            f32::MIN,
            f32::MAX,
            |f| f,
            |v| v
        );
    };
}

macro_rules! impl_compressed_rgba_color_texel_variant {
    ($elem:ty) => {
        internal_impl_color_texel!(
            RGBA,
            $elem,
            Vec4,
            |v: vek::Vec4<Self::Base>| vek::Vec4::from(v),
            0.0,
            1.0,
            |f| (f * u8::MAX as f32) as u8,
            |v| v as f32 / u8::MAX as f32
        );

        internal_impl_color_texel!(
            SRGBA,
            $elem,
            Vec4,
            |v: vek::Vec4<Self::Base>| vek::Vec4::from(v),
            0.0,
            1.0,
            |f| (f * u8::MAX as f32) as u8,
            |v| v as f32 / u8::MAX as f32
        );
    };
}

impl_color_texels!(R, Scalar, |v: vek::Vec4<Self::Base>| v[0]);
impl_color_texels!(RG, Vec2, |v: vek::Vec4<Self::Base>| { vek::Vec2::from(v) });
impl_color_texels!(RGBA, Vec4, |v: vek::Vec4<Self::Base>| {
    vek::Vec4::from(v)
});
internal_impl_color_texel!(
    SRGBA,
    Normalized<u8>,
    Vec4,
    |v: vek::Vec4<Self::Base>| vek::Vec4::from(v),
    0.0,
    1.0,
    |f| (f * u8::MAX as f32) as u8,
    |v| v as f32 / u8::MAX as f32
);
internal_impl_color_texel!(
    BGRA,
    Normalized<u8>,
    Vec4,
    |v: vek::Vec4<Self::Base>| vek::Vec4::from(v),
    0.0,
    1.0,
    |f| (f * u8::MAX as f32) as u8,
    |v| v as f32 / u8::MAX as f32
);
internal_impl_color_texel!(
    SBGRA,
    Normalized<u8>,
    Vec4,
    |v: vek::Vec4<Self::Base>| vek::Vec4::from(v),
    0.0,
    1.0,
    |f| (f * u8::MAX as f32) as u8,
    |v| v as f32 / u8::MAX as f32
);

impl_compressed_rgba_color_texel_variant!(Normalized<UBC1>);
impl_compressed_rgba_color_texel_variant!(Normalized<UBC2>);
impl_compressed_rgba_color_texel_variant!(Normalized<UBC3>);
impl_compressed_rgba_color_texel_variant!(Normalized<UBC7>);

// R<Normalized<UBC4>>
// R<Normalized<SBC4>>
internal_impl_color_texel!(
    R,
    Normalized<UBC4>,
    Scalar,
    |v: vek::Vec4<Self::Base>| v[0],
    0.0,
    1.0,
    |f| (f * u8::MAX as f32) as u8,
    |v| v as f32 / u8::MAX as f32
);
internal_impl_color_texel!(
    R,
    Normalized<SBC4>,
    Scalar,
    |v: vek::Vec4<Self::Base>| v[0],
    -1.0,
    1.0,
    |f| (f * i8::MAX as f32) as i8,
    |v| v as f32 / i8::MAX as f32
);

// RG<Normalized<UBC5>>
// RG<Normalized<SBC5>>
internal_impl_color_texel!(
    RG,
    Normalized<UBC5>,
    Vec2,
    |v: vek::Vec4<Self::Base>| vek::Vec2::from(v),
    0.0,
    1.0,
    |f| (f * u8::MAX as f32) as u8,
    |v| v as f32 / u8::MAX as f32
);
internal_impl_color_texel!(
    RG,
    Normalized<SBC5>,
    Vec2,
    |v: vek::Vec4<Self::Base>| vek::Vec2::from(v),
    -1.0,
    1.0,
    |f| (f * i8::MAX as f32) as i8,
    |v| v as f32 / i8::MAX as f32
);

impl Conversion for Depth<f32>
where
    Self: Texel,
{
    type Target = f32;

    fn try_from_target(target: Self::Target) -> Option<Self::Storage> {
        Some(target)
    }

    fn into_target(texel: Self::Storage) -> Self::Target {
        texel
    }
}

impl Conversion for Depth<Normalized<u16>>
where
    Self: Texel,
{
    type Target = f32;

    fn try_from_target(target: Self::Target) -> Option<Self::Storage> {
        if target > 0.0 && target < 1.0 {
            return Some((target * u16::MAX as f32) as u16);
        } else {
            return None;
        }
    }

    fn into_target(texel: Self::Storage) -> Self::Target {
        texel as f32 / (u16::MAX as f32)
    }
}

impl Conversion for Stencil<u8>
where
    Self: Texel,
{
    type Target = u32;

    fn try_from_target(target: Self::Target) -> Option<Self::Storage> {
        target.try_into().ok()
    }

    fn into_target(texel: Self::Storage) -> Self::Target {
        texel as u32
    }
}
