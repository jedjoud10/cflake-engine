use std::num::NonZeroU32;

// Objects that have a specific and unique OpenGL name, like buffers or textures
pub trait ToGlName {
    fn name(&self) -> NonZeroU32;
}

// Objects that have a specific and unique OpenGL type, like shader sources
pub trait ToGlType {
    fn target(&self) -> u32;
}
