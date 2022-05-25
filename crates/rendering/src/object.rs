use crate::context::Context;
use std::{num::NonZeroU32, rc::Rc, ops::Deref};

// An object that is represented by it's target raw type and it's raw name
pub struct GlObject {
    // The raw OpenGL object ID of the object of this unique type 
    name: Name,

    // The raw OpenGL target
    target: Target,
}

impl GlObject {
    // Create a new object with the raw name and raw target types
    pub(crate) fn from_raw_parts(name: u32, target: u32) -> Self {
        Self { name: Name::Unique(name), target: Target(target) }
    }

    // Get the name wrapper
    pub fn name(&self) -> &Name {
        &self.name
    }
    
    // Get the target wrapper
    pub fn target(&self) -> &Target {
        &self.target
    }
}

// Objects that can be shared/sent to the GPU through OpenGL functions
pub trait Shared: Copy + Sized + Sync + Send {}

// TODO: Manual implementations
impl<T: Copy + Sized + Sync + Send> Shared for T {}


// Copy-on-write reference counted pointer for shared raw OpenGL object
pub enum Name {
    // This is an owned OpenGL object that is not shared
    Unique(gl::types::GLuint),

    // This is a shared OpenGL object
    Shared(Rc<gl::types::GLuint>)
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

impl Deref for Name {
    type Target = gl::types::GLuint;

    fn deref(&self) -> &Self::Target {
        match self {
            Name::Unique(x) => x,
            Name::Shared(x) => x.as_ref(),
        }
    }
}

// A simple OpenGL target wrapper
pub struct Target(gl::types::GLuint);

impl Deref for Target {
    type Target = gl::types::GLuint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}