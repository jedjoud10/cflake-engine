use super::{Depth, Element, Ranged, Stencil, Texel, R, RG, RGB, RGBA, SRGB, SRGBA};

// Color texels are texels that purely represent color data (all texel types other than depth and stencil texels)
pub trait ColorTexel: Texel {}
impl<E: Element> ColorTexel for R<E> where Self: Texel {}
impl<E: Element> ColorTexel for RG<E> where Self: Texel {}
impl<E: Element> ColorTexel for RGB<E> where Self: Texel {}
impl<E: Element> ColorTexel for RGBA<E> where Self: Texel {}
impl<E: Element> ColorTexel for SRGB<E> where Self: Texel {}
impl<E: Element> ColorTexel for SRGBA<E> where Self: Texel {}

// Depth texels are texels that purely represent vertex depth
pub trait DepthTexel: Texel {}
impl<E: Element> DepthTexel for Depth<E> where Self: Texel {}

// Stencil texels are texels that purely represent stencil masks
pub trait StencilTexel: Texel {}
impl<E: Element> StencilTexel for Stencil<E> where Self: Texel {}

// Image texels are texels that can be loaded from a file, like when loading a Texture2D<RGBA<Ranged<u8>>>
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

// SRGBA, u8
impl ImageTexel for SRGBA<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(vek::Vec4::from_slice).collect()
    }
}

// SRGB, u8
impl ImageTexel for SRGB<Ranged<u8>> {
    fn to_image_texels(image: image::DynamicImage) -> Vec<Self::Storage> {
        let image = image.into_rgba8();
        image.chunks(4).map(vek::Vec3::from_slice).collect()
    }
}
