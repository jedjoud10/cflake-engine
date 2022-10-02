use hdrldr::Image as HdrImage;
use image::{DynamicImage, ImageFormat, GenericImageView};

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

// Intermediate file format that we will load inside the ImageTexel trait
// TODO: Somehow remove this sheize
pub enum IntermediateImage {
    Image(DynamicImage),
    Hdr(HdrImage),
}

impl IntermediateImage {
    // Create an intermediate image from it's raw bytes
    pub fn new(bytes: &[u8]) -> Self {
        let guessed = image::guess_format(bytes).unwrap();
            match guessed {
                ImageFormat::Png | ImageFormat::Jpeg => {
                    let loaded = image::load_from_memory_with_format(bytes, guessed).unwrap();
                    loaded.flipv();
                    Self::Image(loaded)
                },
                ImageFormat::Hdr => {
                    let mut loaded = hdrldr::load(bytes).unwrap();
                    // TODO: Optimize this flip bro
                    let rows = loaded.data.chunks(loaded.width as usize);
                    let flipped = rows
                        .rev()
                        .flat_map(|row| row.iter().cloned())
                        .collect::<Vec<hdrldr::RGB>>();
                    loaded.data = flipped;
                    Self::Hdr(loaded)
                },
            _ => panic!("Not tested/supported yet"),
        }
    }

    // Try to get the enum as a dynamic image variant
    pub fn as_image(self) -> Option<DynamicImage> {
        if let Self::Image(image) = self {
            Some(image)
        } else {
            None
        }
    }

    // Try to geth enum as a hdr variant
    pub fn as_hdr(self) -> Option<HdrImage> {
        if let Self::Hdr(hdr) = self {
            Some(hdr)
        } else {
            None
        }
    }

    // Get the resolution of the intermediate image
    pub fn dimensions(&self) -> vek::Extent2<u16> {
        match self {
            IntermediateImage::Image(i) => vek::Extent2::new(i.dimensions().0, i.dimensions().1).as_::<u16>(),
            IntermediateImage::Hdr(i) => vek::Extent2::new(i.width, i.height).as_::<u16>(),
        }
    }
}

// Image texels are texels that can be loaded from a file, like when loading a Texture2D<RGBA<Ranged<u8>>>
pub trait ImageTexel: Texel {
    // Given a loaded file format, we must decode it to it's raw texels
    fn read(loaded: IntermediateImage) -> Vec<Self::Storage>;
}

impl ImageTexel for R<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba8().chunks(4).map(|val| val[0]).collect()
    }
}

impl ImageTexel for RG<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba8().chunks(4).map(vek::Vec2::from_slice).collect()
    }
}

impl ImageTexel for RGB<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgb8().chunks(3).map(vek::Vec3::from_slice).collect()
    }
}

impl ImageTexel for RGBA<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba8().chunks(4).map(vek::Vec4::from_slice).collect()
    }
}

impl ImageTexel for SRGBA<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba8().chunks(4).map(vek::Vec4::from_slice).collect()
    }
}

impl ImageTexel for SRGB<Ranged<u8>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgb8().chunks(4).map(vek::Vec3::from_slice).collect()
    }
}

impl ImageTexel for R<Ranged<u16>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba16().chunks(4).map(|val| val[0]).collect()
    }
}

impl ImageTexel for RG<Ranged<u16>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba16().chunks(4).map(vek::Vec2::from_slice).collect()
    }
}

impl ImageTexel for RGB<Ranged<u16>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgb16().chunks(3).map(vek::Vec3::from_slice).collect()
    }
}

impl ImageTexel for RGBA<Ranged<u16>> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_image().unwrap().into_rgba16().chunks(4).map(vek::Vec4::from_slice).collect()
    }
}

impl ImageTexel for RGB<f32> {

    fn read(loaded: IntermediateImage) -> Vec<Self::Storage> {
        loaded.as_hdr().unwrap().data.into_iter().map(|rgb| vek::Vec3::new(rgb.r, rgb.g, rgb.b)).collect()
    }
}
