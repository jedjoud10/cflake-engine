use std::num::NonZeroU32;

use super::Context;

// Objects that have a specific and unique OpenGL name, like buffers or textures
pub trait ToGlName {
    fn name(&self) -> NonZeroU32;
}

// Objects that have a specific and unique OpenGL type, like shader sources
pub trait ToGlType {
    fn target(&self) -> u32;
}

// This will be implemented for OpenGL objects that can be bound
pub trait Bind {
    // Bind an object so we can update/modify it
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>));
}

// This implies that the internal object is a bound OpenGL object that we can modify
pub struct Active<'a, T>(pub(crate)&'a mut T);

impl<'a, T: ToGlType> ToGlType for Active<'a, T> {
    fn target(&self) -> u32 {
        self.0.target()
    }
}

impl<'a, T: ToGlName> ToGlName for Active<'a, T> {
    fn name(&self) -> NonZeroU32 {
        self.0.name()
    }
}