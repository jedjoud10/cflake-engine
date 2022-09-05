use super::{
    ColorAttachmentLayout, DepthAttachment, PainterColorLayout, PainterDepthTexel,
    PainterStencilTexel, StencilAttachment, UntypedAttachment,
};
use crate::{
    context::Context,
    display::{Display, Viewport},
    prelude::{Texel, TexelFormat},
};
use std::marker::PhantomData;

// A painter is a safe wrapper around an OpenGL framebuffer
// However, a painter by itself does not store textures / renderbuffers
// A painter must be "used" to give us a scoped painter that we can use to set targets
// These targets are the actual textures / render buffers that we wish to draw to
pub struct Painter<C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> {
    pub(super) name: u32,
    attachments: Vec<UntypedAttachment>,
    _phantom: PhantomData<*const C>,
    _phantom2: PhantomData<*const D>,
    _phantom3: PhantomData<*const S>,
}

impl<C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> Painter<C, D, S> {
    // Create a new painter using the graphics context
    pub fn new(ctx: &mut Context) -> Self {
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            name
        };
        
        Self {
            name,
            attachments: Vec::new(),
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _phantom3: PhantomData,
        }
    }

    // Use the painter to give us a scoped painter that has proper targets
    // This will return None if the current painter is defined as empty
    pub fn scope<CT: ColorAttachmentLayout<C>, DT: DepthAttachment<D>, ST: StencilAttachment<S>>(
        &mut self,
        viewport: Viewport,
        colors: CT,
        depth: DT,
        stencil: ST,
    ) -> Option<ScopedPainter<C, D, S>> {
        todo!()
    }

    /*
    // Attach a new target to the canvas using it's binding (only if necessary)
    pub(super) unsafe fn attach<T: Texel>(&mut self, storage: Attachment<T>, binding: &mut AttachmentBinding<T>) {
        // We cannot use the binding from another canvas
        if binding.framebuffer() != self.name {
            panic!("Cannot use the given binding since it originates from another canvas");
        }

        // Check if we *must* send out OpenGL commands
        if (self.layout & (1 >> binding.index())) == 1 {
            if binding.attachment == Some(storage) {
                return;
            } else {
                binding.attachment = Some(storage);
            }
        }

        // Set the framebuffer storage
        match storage {
            Attachment::TextureLevel { texture_name, level, layer, .. } => {
                gl::NamedFramebufferTextureLayer(self.name, binding.location(), texture_name, level as i32, layer as i32);
            },
        }
    }
    */
}

// A scoped painter is what we must use to be able to use the Display's trait functionality
pub struct ScopedPainter<'a, C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> {
    painter: &'a mut Painter<C, D, S>,
    viewport: Viewport,
}

impl<C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> Display
    for ScopedPainter<'_, C, D, S>
{
    fn viewport(&self) -> crate::display::Viewport {
        self.viewport
    }

    fn name(&self) -> u32 {
        self.painter.name
    }
}
