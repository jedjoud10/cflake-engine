use super::Texture;


// A sampler is used as an interface between textures and Shaders. We can use samplers to read textures within shaders, and each texture has a unique sampler associated with it
// For now, samplers are just simple wrappers around textures. They just help organizing and separating "textures" from actual shader "samplers"
pub struct Sampler<'a, T: Texture>(pub(crate) &'a T);
