use std::marker::PhantomData;

use ahash::AHashMap;
use crate::{prelude::{Texel, Texture2D, MipLayerMut, TexelFormat, UntypedTexel}, context::{ToGlName, Context}};

// This is the target for a specific framebuffer attachment
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum CanvasAttachmentLocation {
    Color(u32),
    Depth,
    Stencil,
}

// This is a wrapper around framebuffer attachments
// These values will be stored within the canvas
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CanvasAttachment {
    TextureLayer2D {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel
    },
}

// This trait is implemented for Textures and RenderBuffers that can be converted to CanvasStorage
pub trait ToCanvasAttachment<'a> {
    fn into(&self) -> CanvasAttachment;
}

// Will use the mip level given by the layer
impl<'a, T: Texel> ToCanvasAttachment<'a> for MipLayerMut<'a, Texture2D<T>> {
    fn into(&self) -> CanvasAttachment {
        CanvasAttachment::TextureLayer2D {
            texture_name: self.texture().name(),
            level: self.level(),
            untyped: T::untyped(),
        }
    }
}


// A painter is a safe wrapper around an OpenGL framebuffer
// However, a painter by itself does not store textures / renderbuffers
// A painter must be "used" to give us a scoped painter that we can use to set targets
// These targets are the actual textures / render buffers that we wish to draw to
pub type AttachedStorages = AHashMap<CanvasAttachmentLocation, CanvasAttachment>;
pub struct Painter {
    name: u32,
    attachments: AttachedStorages,
    _phantom: PhantomData<*const u32>,
}

impl Painter {    
    // Create a new blank framebuffer with no layout
    pub fn new(ctx: &mut Context) -> Self {
        let mut name = 0u32;
        unsafe {
            gl::GenFramebuffers(1, &mut name);
        }

        Self {
            name,
            attachments: Default::default(),
            _phantom: PhantomData::default(),
        }
    }

    // Attach a new target to the painter (if possible)
    unsafe fn attach(&mut self, storage: CanvasAttachment) -> bool {
        // Check if we have depth/stencil/color attachments already
        let depth = self.is_attached(CanvasAttachmentLocation::Depth);

        let target = match storage {
            CanvasAttachment::TextureLayer2D { texture_name, level, untyped } => match untyped.enum_format {
                TexelFormat::Color | TexelFormat::GammaCorrectedColor => CanvasAttachmentLocation::Color(()),
                TexelFormat::Depth => CanvasAttachmentLocation::Depth,
                TexelFormat::Stencil => CanvasAttachmentLocation::Stencil,
            },
        }
    }

    // Get a list of all the currently attached storages
    pub fn attachments(&self) -> &AttachedStorages {
        &self.attachments
    }

    // Check if the canvas uses a depth target
    pub fn is_depth_enabled(&self) -> bool {
        self.attachments.contains_key(&CanvasAttachmentLocation::Depth)
    }

    // Check if the canvas uses a stencil target
    pub fn is_stencil_enabled(&self) -> bool {
        self.attachments.contains_key(&CanvasAttachmentLocation::Stencil)
    }

    // Check if the canvas uses a color target, and if so, return it's last location index
    pub fn is_color_enabled(&self) -> Option<u32> {
        self.attachments.iter().fold(None, |acc, (item, _)| match item {
            CanvasAttachmentLocation::Color(new) => match acc {
                Some(old) => Some(old.max(*new)),
                None => Some(*new),
            },
            _ => acc,
        })
    }
}


// A scoped painter will be used to set the render targets that we wish to draw to
pub struct ScopedPainter<'a> {
    painter: &'a mut Painter,
    current: AttachedStorages,
}

impl<'a> ScopedPainter<'a> {
    // Create a new scoped painter from a backing painter
    pub fn new(painter: &mut Painter) -> Self {
        Self {
            painter,
            current: Default::default(),
        }
    }

    // Set a specific storage target inside the painter
    pub fn set_target<'b, T: ToCanvasAttachment<'b>>(&mut self, target: T) -> Option<()> {
        unsafe { self.painter.attach(T::into(&target)) }
    }
}