use crate::context::Context;
use std::{num::NonZeroU32, rc::Rc, ops::Deref};
// This trait will be implemented on objects that contain a raw OpenGL name
pub trait ToGlName {
    fn name(&self) -> u32;
}

// This trait will be implemented on objects that have a unique OpenGL type
pub trait ToGlTarget {
    fn target() -> u32;
}

// Objects that can be shared/sent to the GPU through OpenGL functions
pub trait Shared: Copy + Sized + Sync + Send {}

// TODO: Manual implementations
impl<T: Copy + Sized + Sync + Send> Shared for T {}