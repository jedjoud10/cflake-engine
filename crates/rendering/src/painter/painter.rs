use std::marker::PhantomData;

use ahash::AHashSet;

use crate::{prelude::{TexelFormat, Texel, Texture2D, MipLayerMut}, display::{Viewport, Display}, context::Context};
use super::{CanvasStorage, ToCanvasStorage, CanvasLayout};

// This painter is an abstraction over OpenGL framebuffers
// Painters by themselves don't contain the storages attachments, but they contain the formats of the last used attachments 
pub struct Painter {
    framebuffer: u32,
    viewport: Viewport,
    storages: Vec<CanvasStorage>,
}

impl Painter {
    // Create a new framebuffer wrapper with no attachments
    pub fn new(ctx: &mut Context, viewport: Viewport) -> Self {
        // Create a new framebuffer 
        let mut name = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut name);
        }

        Self {
            framebuffer: name,
            viewport,
            storages: Vec::new(),
        }
    }

    // Get the viewport from this painter
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    // Update the painting viewport
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    // Get the viewport extent of this painter
    pub fn size(&self) -> vek::Extent2<u16> {
        self.viewport().extent
    } 
}

impl Drop for Painter {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.framebuffer);
        }
    }
}


// A scoped painter is what we will use to be able to write to a canvas
// We can set the textures that we will write to using this scoped painter
pub struct ScopedPainter<'layout, 'painter, L: CanvasLayout<'layout>> {
    painter: &'painter mut Painter,
    phantom_: PhantomData<&'layout mut L>, 
}

impl<'layout, 'painter, L: CanvasLayout<'layout>> ScopedPainter<'layout, 'painter, L> {
    // Create a new scoped canvas from a painter and a canvas layout 
    pub fn new(painter: &'painter mut Painter, layout: L) -> Option<Self> {
        // Make sure the layout is valid
        // Update the underlying framebuffer if needed
        None
    }
}

impl<'layout, 'painter, L: CanvasLayout<'layout>> Display for ScopedPainter<'layout, 'painter, L> {
    fn viewport(&self) -> Viewport {
        self.painter.viewport()
    }

    fn name(&self) -> u32 {
        self.painter.framebuffer
    }
}