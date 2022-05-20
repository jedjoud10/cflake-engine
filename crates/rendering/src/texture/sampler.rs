use std::marker::PhantomData;

use super::Texture;

// A sampler is the interface between Textures and Shaders. Samplers allow us to read textures within shaders
pub struct Sampler {
}