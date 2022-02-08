use ahash::AHashMap;

use crate::basics::uniforms::UniformsDefinitionMap;

// Stores everything that needs to be cached for rendering
#[derive(Default)]
pub struct Cached {
    // Uniform definitions for each shader program
    pub(crate) uniform_defitions: AHashMap<u32, UniformsDefinitionMap>,
}