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

// Objects that can be shared/sent to the GPU through OpenGL functions
// This represents a SINGLE value, not multiple, so no slices / arrays
pub trait Shared: Copy + Sized + Sync + Send {}

// Manual implementations


// This implies that the internal object is a bound OpenGL object that we can modify
pub struct Active<'a, T> {
    pub(crate) inner: &'a mut T,
    pub(crate) context: &'a mut Context, 
}

impl<'a, T: ToGlType> ToGlType for Active<'a, T> {
    fn target(&self) -> u32 {
        self.inner.target()
    }
}

impl<'a, T: ToGlName> ToGlName for Active<'a, T> {
    fn name(&self) -> NonZeroU32 {
        self.inner.name()
    }
}