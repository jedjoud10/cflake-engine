use crate::Texel;

// Color texels are texels used for color attachments
// TODO: Figure out if there are any limits to this
// TODO: Maybe add saturation / hue control (since this is strictly color after all)
pub trait ColorTexel: Texel {
    // Convert an RGBA f32 value to the color texel
    // Returns None if the conversion fails
    // If the ColorTexel contains less channels than the Vec4, the extra channels are zeroed out
    fn try_from_rgba32(rgba: vek::Vec4<f32>) -> Option<Self::Storage>;

    // Converts the color texel to an RGBA f32 value
    // If the ColorTexel contains less channels than the Vec4, the extra channels are discarded
    fn to_rgba32(texel: Self::Storage) -> vek::Vec4<f32>;

    // Convert this texel to a color value
    fn into_color(texel: Self::Storage) -> wgpu::Color {
        let rgba = Self::to_rgba32(texel);

        wgpu::Color {
            r: rgba.x as f64,
            g: rgba.y as f64,
            b: rgba.z as f64,
            a: rgba.w as f64,
        }
    }
}

macro_rules! internal_impl_color_texel {
    ($vec:ident, $elem:ty, $channels:expr, $storagevec: ident) => {
        impl ColorTexel for $vec<$elem> {
        }
    };
}

macro_rules! impl_color_texels {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_texel!($vec, u8, $channels, $storagevec);
        internal_impl_texel!($vec, i8, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<u8>, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<i8>, $channels, $storagevec);

        internal_impl_texel!($vec, u16, $channels, $storagevec);
        internal_impl_texel!($vec, i16, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<u16>, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<i16>, $channels, $storagevec);

        internal_impl_texel!($vec, u32, $channels, $storagevec);
        internal_impl_texel!($vec, i32, $channels, $storagevec);

        internal_impl_texel!($vec, f16, $channels, $storagevec);
        internal_impl_texel!($vec, f32, $channels, $storagevec);
    };  
}

