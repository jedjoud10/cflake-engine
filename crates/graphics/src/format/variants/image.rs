use crate::{Texel, ColorTexel, RG, RGBA, Normalized, R};

// Image texels are texels that can be loaded from a file, like when loading a Texture2D<RGBA<Normalized<u8>>
pub trait ImageTexel: Texel + ColorTexel {
    fn to_image_texels(
        image: image::DynamicImage,
    ) -> Vec<Self::Storage>;
}

// Internally used for implementing the image texel
macro_rules! internal_impl_single_image_texel {
    ($t:ident, $base:ty, $convert:ident, $closure:expr) => {
        impl ImageTexel for $t<$base> {
            fn to_image_texels(
                image: image::DynamicImage,
            ) -> Vec<Self::Storage> {
                let image = image.$convert();
                image.chunks(4).map($closure).collect()
            }
        }
    };
}

// Implement the image texel layouts
macro_rules! impl_image_texel {
    ($t:ident, $closure:expr) => {
        internal_impl_single_image_texel!(
            $t, u8, into_rgba8, $closure
        );
        internal_impl_single_image_texel!(
            $t,
            u16,
            into_rgba16,
            $closure
        );
        internal_impl_single_image_texel!(
            $t,
            Normalized<u8>,
            into_rgba8,
            $closure
        );
        internal_impl_single_image_texel!(
            $t,
            Normalized<u16>,
            into_rgba16,
            $closure
        );
    };
}

impl_image_texel!(R, |val| val[0]);
impl_image_texel!(RG, vek::Vec2::from_slice);
impl_image_texel!(RGBA, vek::Vec4::from_slice);