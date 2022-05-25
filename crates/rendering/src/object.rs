use crate::context::Context;
use std::{num::NonZeroU32, rc::Rc, ops::Deref};
pub(crate) use raw::*;
mod raw {
    // This trait will be implemented on objects that contain a raw OpenGL name
    pub trait ToGlName {
        fn name(&self) -> u32;
    }

    // This trait will be implemented on objects that have a unique OpenGL type
    pub trait ToGlTarget {
        fn target() -> u32;
    }
}

// Objects that can be shared/sent to the GPU through OpenGL functions
pub trait Shared: Copy + Sized + Sync + Send {}

// TODO: Manual implementations
impl<T: Copy + Sized + Sync + Send> Shared for T {}


// Copy-on-write reference counted pointer for shared raw OpenGL object names
pub enum Name {
    // This is an owned OpenGL object that is not shared
    Unique(u32),

    // This is a shared OpenGL object
    Shared(Rc<u32>)
}

impl Name {
    // Check if the name is shared between multiple objects
    pub fn is_shared(&self) -> bool {
        if let Name::Shared(_) = self {
            true
        } else { false }
    }
    
    // Check if the name is unique for the current object
    pub fn is_unique(&self) -> bool {
        if let Name::Unique(_) = self {
            true
        } else { false }
    }
}

impl From<u32> for Name {
    fn from(name: u32) -> Self {
        Name::Unique(name)
    }
}

impl Deref for Name {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Name::Unique(x) => x,
            Name::Shared(x) => x.as_ref(),
        }
    }
}