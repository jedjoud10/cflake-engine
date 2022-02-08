use ahash::AHashMap;
use crate::basics::uniforms::{ShaderIDType, UniformsDefinitionMap};

// Stores everything that needs to be cached for rendering
pub struct Cached {
    // Shaders and uniforms
    uniform_defitions: AHashMap<ShaderIDType, UniformsDefinitionMap>,
}