use crate::context::Context;
use std::num::NonZeroU32;

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
pub trait Shared: Copy + Sized + Sync + Send {}

// TODO: Manual implementations
impl<T: Copy + Sized + Sync + Send> Shared for T {}

// This implies that the internal object is a bound OpenGL object that we can modify
pub struct Active<'a, T> {
    inner: &'a mut T,
}

impl<'a, T> Active<'a, T> {
    // Create a bound object, indeed
    pub fn new(obj: &'a mut T, _ctx: &mut Context) -> Self {
        Self {
            inner: obj
        }
    }
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

impl<'a, T> AsRef<T> for Active<'a, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}

impl<'a, T> AsMut<T> for Active<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.inner
    }
}