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
pub trait Bind: ToGlType + ToGlName + Sized {
    // This will check if the current object is bound into the context
    fn is_bound(&self, ctx: &Context) -> bool {
        ctx.bound.get(&self.target()).map(|name| *name == self.name().get()).unwrap_or_default()
    }

    // This must always bind the object, This is used internally, so I should probably just hide it
    unsafe fn bind_raw_unchecked(&mut self, ctx: &mut Context);

    // Bind an object, but make sure that it is already unbounded first
    fn bind(&mut self, ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        if !self.is_bound(ctx) {
            unsafe { self.bind_raw_unchecked(ctx) };
            function(Active(self));
        }
    }
}

// Objects that can be shared/sent to the GPU through OpenGL functions
pub trait Shared: Copy + Sized + Sync + Send {}

// TODO: Manual implementations
impl<T: Copy + Sized + Sync + Send> Shared for T {}

// This implies that the internal object is a bound OpenGL object that we can modify
pub struct Active<'a, T>(&'a mut T);

impl<'a, T> AsRef<T> for Active<'a, T> {
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<'a, T> AsMut<T> for Active<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.0
    }
}
