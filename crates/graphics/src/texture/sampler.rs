use std::marker::PhantomData;

use crate::Texel;

// A sampler is a special objects that allows us to read textures from within shaders
// Samplers are not "connected" to specific textures, and you can create samplers by themselves
pub struct Sampler<T: Texel> {
    _phantom: PhantomData<T>,
}