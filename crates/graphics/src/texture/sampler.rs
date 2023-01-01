use std::marker::PhantomData;
use vulkan::vk;
use crate::{Texel, Graphics};

// A sampler is a special objects that allows us to read textures from within shaders
// Samplers are not "connected" to specific textures, and you can create samplers by themselves
pub struct Sampler<T: Texel> {
    _phantom: PhantomData<T>,
    sampler: vk::Sampler,
}

impl<T: Texel> Drop for Sampler<T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<T: Texel> Sampler<T> {
    // Create a new sampler for a specific texel type
    pub fn new(graphics: &Graphics,) -> Self {
        let format = T::FORMAT;

        todo!()
    }
}