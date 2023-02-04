use crate::{Texel, ColorTexel};

// Image texels are texels that can be loaded from a file, like when loading a Texture2D<RGBA<Normalized<u8>>
pub trait ImageTexel: Texel + ColorTexel {
    fn to_image_texels(
        image: image::DynamicImage,
    ) -> Vec<Self::Storage>;
}