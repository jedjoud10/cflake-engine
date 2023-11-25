use crate::{texture::Extent, context::Graphics, format::{ImageTexel, Texel}};
use assets::Asset;
pub use image::imageops::FilterType;
use image::ImageFormat;
use thiserror::Error;

// Texture resolution scale that we can use to downsample or upsample imported textures
#[derive(Default, Copy, Clone, PartialEq)]
pub enum TextureScale {
    // This will not affect the texture scale
    #[default]
    Default,

    // This will scale the texture size with the "scaling" parameter
    Scale {
        scaling: f32,
        filter: FilterType,
    },
}

// Type of error that might occur when deserializing an image
#[derive(Debug, Error)]
pub enum RawTexelsError {
    #[error("{0}")]
    ImageDeserialization(image::ImageError),

    #[error("{0:?}")]
    HdrDeserialization(hdrldr::LoadError),
}

// Represents an intermediate texture that contains raw texels from a file
pub struct RawTexels<T: ImageTexel>(pub Vec<T::Storage>, pub vek::Extent2<u32>);

impl<T: ImageTexel> RawTexels<T> {
    // Get the underlying texels immutably
    pub fn texels(&self) -> &[T::Storage] {
        &self.0
    }

    // Get the underlying texels mutably
    pub fn texels_mut(&mut self) -> &mut [T::Storage] {
        &mut self.0
    }

    // Get the dimensions of the raw texels
    pub fn dimensions(&self) -> vek::Extent2<u32> {
        self.1
    }
}

impl<T: ImageTexel> Asset for RawTexels<T> {
    type Context<'ctx> = ();
    type Settings<'stg> = TextureScale;
    type Err = RawTexelsError;

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        let bytes = data.bytes();
        let guessed = image::guess_format(bytes).unwrap();

        // Deserialize, scale down/up, and convert to texels
        let (texels, extent) = match guessed {
            // Load PNG and JPEG using image crate
            ImageFormat::Png | ImageFormat::Jpeg => {
                let mut image = image::load_from_memory_with_format(bytes, guessed)
                    .map_err(RawTexelsError::ImageDeserialization)?;

                // Scale the texture if needed
                if let TextureScale::Scale { scaling, filter } = settings {
                    let nheight = ((image.height() as f32) * scaling) as u32;
                    let nwidth = ((image.width() as f32) * scaling) as u32;

                    if nheight != 0 && nwidth != 0 {
                        image = image.resize(nwidth, nheight, filter);
                    }
                }

                let extent = vek::Extent2::new(image.width(), image.height());
                let texels = T::dyn_image_to_texels(image);
                (texels.unwrap(), extent)
            }

            // Load HDR using specialized crate
            ImageFormat::Hdr => {
                /*
                let mut loaded = hdrldr::load(bytes).unwrap();
                // TODO: Optimize this flip bro
                let rows = loaded.data.chunks(loaded.width as usize);
                let flipped = rows
                    .rev()
                    .flat_map(|row| row.iter().cloned())
                    .collect::<Vec<hdrldr::RGB>>();
                loaded.data = flipped;
                Self::Hdr(loaded)
                */
                todo!()
            }

            _ => panic!("Not supported yet"),
        };

        return Ok(Self(texels, extent));
    }
}
