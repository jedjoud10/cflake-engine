use super::{Ranged, Texel, R, RG, RGB, RGBA};

// Image texels are texels that can be loaded from a .png file, like when loading a Texture2D<RGBA<Ranged<u8>>>
pub trait ImageTexel: Texel {
    // Fetch the image texels from a loaded dynamic image stored on disk (or embedded into the binary)
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage>;
}

// Red channel only, u8
impl ImageTexel for R<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(|val| val[0]).collect()
    }
}

// Red-Green channels only, u8
impl ImageTexel for RG<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(vek::Vec2::from_slice).collect()
    }
}

// Red-green-blue channels only, u8
impl ImageTexel for RGB<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(vek::Vec3::from_slice).collect()
    }
}

// RGBA, u8
impl ImageTexel for RGBA<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(vek::Vec4::from_slice).collect()
    }
}
