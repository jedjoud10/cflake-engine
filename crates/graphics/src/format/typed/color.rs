use crate::{Texel, Normalized, RG, RGBA, BGRA, R, AnyElement};
use half::f16;

// Color texels are texels used for color attachments
// TODO: Figure out if there are any limits to this
// TODO: Maybe add saturation / hue control (since this is strictly color after all)
pub trait ColorTexel: Texel {
    // Convert an RGBA f32 v to the color texel
    // Returns None if the conversion fails
    // If the ColorTexel contains less channels than the Vec4, the extra channels are zeroed out
    fn try_from_rgba32(rgba: vek::Vec4<f32>) -> Option<Self::Storage>;

    // Converts the color texel to an RGBA f32 v
    // If the ColorTexel contains less channels than the Vec4, the extra channels are discarded
    fn to_rgba32(texel: Self::Storage) -> vek::Vec4<f32>;

    // Convert this texel to a color v (if possible)
    // This returns None if the color data is not in the 0 - 1 range 
    fn into_color(texel: Self::Storage) -> Option<wgpu::Color> {
        let rgba = Self::to_rgba32(texel);

        if rgba.reduce_partial_max() > 1.0 || rgba.reduce_partial_min() < 0.0 {
            return None;
        }

        Some(wgpu::Color {
            r: rgba.x as f64,
            g: rgba.y as f64,
            b: rgba.z as f64,
            a: rgba.w as f64,
        })
    }
}


fn map<T: Texel>(rgba: vek::Vec4<f32>, map: impl Fn(f32) -> T::Base) -> vek::Vec4<T::Base> {
    rgba.map(map)
}

// Do NOT FUCKING TOUCH THIS
macro_rules! internal_impl_color_texel {
    ($vec:ident, $elem:ty, $channels:expr, $storagevec:ident, $conv:tt, $min:expr, $max:expr, $fromf32:expr, $tof32:expr) => {
        impl ColorTexel for $vec<$elem> {
            fn try_from_rgba32(rgba: vek::Vec4<f32>) -> Option<Self::Storage> {
                if rgba.reduce_partial_max() > $max || rgba.reduce_partial_min() < $min {
                    return None;
                }

                let converted = map::<Self>(rgba, $fromf32); 
                let killme = $conv;
                let storage = killme(converted);
                Some(storage)
            }

            fn to_rgba32(texel: Self::Storage) -> vek::Vec4<f32> {
                let input = vek::Vec4::<Self::Base>::from(texel);
                let mapped = input.map($tof32);
                mapped
            }
        }
    };
}

// I deserve to be in the deepest layers of hell
macro_rules! impl_color_texels {
    ($vec:ident, $channels:expr, $storagevec:ident, $conv:expr) => {
        internal_impl_color_texel!($vec, u8, $channels, $storagevec, $conv, u8::MIN as f32, u8::MAX as f32, |f| f as u8, |v| v as f32);
        internal_impl_color_texel!($vec, i8, $channels, $storagevec, $conv, i8::MIN as f32, i8::MAX as f32, |f| f as i8, |v| v as f32);
        internal_impl_color_texel!($vec, Normalized<u8>, $channels, $storagevec, $conv, 0.0, 1.0, |f| (f * u8::MAX as f32) as u8, |v| v as f32 / u8::MAX as f32);
        internal_impl_color_texel!($vec, Normalized<i8>, $channels, $storagevec, $conv, -1.0, 1.0, |f| (f * i8::MAX as f32) as i8, |v| v as f32 / i8::MAX as f32);

        internal_impl_color_texel!($vec, u16, $channels, $storagevec, $conv, u16::MIN as f32, u16::MAX as f32, |f| f as u16, |v| v as f32);
        internal_impl_color_texel!($vec, i16, $channels, $storagevec, $conv, i16::MIN as f32, i16::MAX as f32, |f| f as i16, |v| v as f32);
        internal_impl_color_texel!($vec, Normalized<u16>, $channels, $storagevec, $conv, 0.0, 1.0, |f| (f * u16::MAX as f32) as u16, |v| v as f32 / u16::MAX as f32);
        internal_impl_color_texel!($vec, Normalized<i16>, $channels, $storagevec, $conv, -1.0, 1.0, |f| (f * i16::MAX as f32) as i16, |v| v as f32 / i16::MAX as f32);

        internal_impl_color_texel!($vec, u32, $channels, $storagevec, $conv, u32::MIN as f32, u32::MAX as f32, |f| f as u32, |v| v as f32);
        internal_impl_color_texel!($vec, i32, $channels, $storagevec, $conv, i32::MIN as f32, i32::MAX as f32, |f| f as i32, |v| v as f32);

        internal_impl_color_texel!($vec, f16, $channels, $storagevec, $conv, f32::from(f16::MIN), f32::from(f16::MAX), |f| f16::from_f32(f), |v| f32::from(v));
        internal_impl_color_texel!($vec, f32, $channels, $storagevec, $conv, f32::MIN, f32::MAX, |f| f, |v| v);
    };  
}

type Scalar<T> = T;
impl_color_texels!(R, ChannelsType::Vector(VectorChannels::One), Scalar, |v: vek::Vec4<Self::Base>| v[0]);
impl_color_texels!(RG, ChannelsType::Vector(VectorChannels::Two), Vec2, |v: vek::Vec4<Self::Base>| vek::Vec2::from(v));
impl_color_texels!(RGBA, ChannelsType::Vector(VectorChannels::Four), Vec4, |v: vek::Vec4<Self::Base>| vek::Vec4::from(v));
//internal_impl_color_texel!(BGRA, Normalized<u8>, ChannelsType::Vector(VectorChannels::FourSwizzled), Vec4, from_slice, 0.0, 1.0, |f| (f * u8::MAX as f32) as u8, |v| v as f32 / u8::MAX as f32);

