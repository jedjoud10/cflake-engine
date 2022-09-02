use std::{marker::PhantomData, mem::MaybeUninit};

use ahash::AHashMap;
use crate::{prelude::{Texel, Texture2D, MipLevelMut, TexelFormat, UntypedTexel}, context::{ToGlName, Context}, display::{Display, Viewport}};

// This is the target for a specific framebuffer attachment
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum AttachmentLocation {
    Color(u32),
    Depth,
    Stencil,
}

// An attachment binding is a wrapper around an attachment location that was accepted into the canvas
pub struct AttachmentBinding<T: Texel> {
    converted_gl_location: u32,
    index: usize,
    _phantom: PhantomData<T>,
}

// This is a wrapper around framebuffer attachments
// These values will be stored within the canvas
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FramebufferAttachment<T: Texel> {
    TextureLevel {
        texture_name: u32,
        level: u8,
        layer: u8,
        _phantom: PhantomData<T>,
    },
}


// This trait is implemented for Textures and RenderBuffers that can be converted to CanvasStorage
pub trait ToFramebufferAttachment<'a, T: Texel> {
    fn into(&self) -> FramebufferAttachment<T>;
}

// Will use the mip level given by the level
impl<'a, T: Texel> ToFramebufferAttachment<'a, T> for MipLevelMut<'a, Texture2D<T>> {
    fn into(&self) -> FramebufferAttachment<T> {
        FramebufferAttachment::TextureLevel {
            texture_name: self.texture().name(),
            level: self.level(),
            layer: 0,
            _phantom: Default::default(),
        }
    }
}


// A painter is a safe wrapper around an OpenGL framebuffer
// However, a painter by itself does not store textures / renderbuffers
// A painter must be "used" to give us a scoped painter that we can use to set targets
// These targets are the actual textures / render buffers that we wish to draw to
pub struct Canvas {
    name: u32,
    layout: u32, // Depth is the first bit on the right, stencil is the second bit on the right
    _phantom: PhantomData<*const u32>,
}

impl Canvas {    
    // Create a new blank framebuffer with a specific layout
    pub fn new(ctx: &mut Context) -> (Self, L::Out) {
        let mut name = 0u32;
        unsafe {
            gl::GenFramebuffers(1, &mut name);
        }

        Self {
            name,
            layout: 0,
            _phantom: PhantomData,
        }
    }

    // Create a new attachment binding for a canvas storage of a specific texel type
    pub fn insert<T: Texel>(&mut self) -> AttachmentBinding<T> {
        todo!()
    }

    // Attach a new target to the painter (only if necessary)
    unsafe fn attach<T: Texel>(&mut self, storage: FramebufferAttachment<T>, binding: &AttachmentBinding<T>) {
        // Check if we *must* send out OpenGL commands
        if (self.layout & (1 >> binding.index)) == 1 {
            let attachment = *self.bindings[binding.index].1.assume_init_ref();
            if attachment == storage {
                return;
            }
        }

        // Set the framebuffer storage
        match storage {
            FramebufferAttachment::TextureLevel { texture_name, level, layer, untyped } => {
                gl::NamedFramebufferTextureLayer(self.name, binding.converted_gl_location, texture_name, level as i32, layer as i32);
            },
        }
    }

    // Check if the canvas uses a depth target
    pub fn is_depth_enabled(&self) -> bool {
        (self.layout & 1) == 1
    }

    // Check if the canvas uses a stencil target
    pub fn is_stencil_enabled(&self) -> bool {
        (self.layout & 2) == 1
    }

    // Check if we have ANY color attachments used
    pub fn color_attachments_enabled(&self) -> bool {
        self.layout > 2
    }

    // Get the maximum color attachment index
    pub fn max_color_attachment_index(&self) -> u32 {
        (32 - self.layout.leading_zeros()) - 3
    }
}


// A scoped painter will be used to set the render targets that we wish to draw to
pub struct ScopedPainter<'a> {
    canvas: &'a mut Canvas,
    viewport: Viewport,
    max_color_attachment: Option<u32>,
}

impl<'a> ScopedPainter<'a> {
    // Create a new scoped painter from a backing painter
    pub fn new(painter: &'a mut Canvas, viewport: Viewport) -> Self {
        Self {
            canvas: painter,
            viewport,
            max_color_attachment: None,
        }
    }

    // Set a specific storage target inside the painter
    pub fn set_rw_target<'b, T: Texel, F: ToFramebufferAttachment<'b, T>>(&mut self, target: F, binding: &AttachmentBinding<T>) {
        unsafe { self.canvas.attach(T::into(&target), binding) }
    }
}

impl Display for ScopedPainter<'_> {
    fn viewport(&self) -> crate::display::Viewport {
        self.viewport
    }

    fn name(&self) -> u32 {
        self.canvas.name
    }
}