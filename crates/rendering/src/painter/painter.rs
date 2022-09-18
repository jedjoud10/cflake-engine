use itertools::Itertools;

use super::{
    ColorAttachmentLayout, DepthAttachment, PainterColorLayout, PainterDepthTexel,
    PainterStencilTexel, StencilAttachment, UntypedAttachment,
};
use crate::{
    context::Context,
    display::{Display, Viewport},
    prelude::{Texel},
};
use std::marker::PhantomData;

// A painter is a safe wrapper around an OpenGL framebuffer
// Painters only store the texel types that we shall use, but they do not store the attachments by themselves
pub struct Painter<C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> {
    pub(super) name: u32,
    untyped_color_attachments: Option<Vec<UntypedAttachment>>,
    untyped_depth_attachment: Option<UntypedAttachment>,
    untyped_stencil_attachment: Option<UntypedAttachment>,
    _phantom: PhantomData<*const C>,
    _phantom2: PhantomData<*const D>,
    _phantom3: PhantomData<*const S>,
}

impl<C: PainterColorLayout, D: PainterDepthTexel, S: PainterStencilTexel> Painter<C, D, S> {
    // Create a new painter using the graphics context
    pub fn new(_ctx: &mut Context) -> Self {
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            name
        };

        Self {
            name,
            untyped_color_attachments: None,
            untyped_depth_attachment: None,
            untyped_stencil_attachment: None,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _phantom3: PhantomData,
        }
    }

    // Use the painter to give us a scoped painter that has proper targets
    pub fn scope<CT: ColorAttachmentLayout<C>, DT: DepthAttachment<D>, ST: StencilAttachment<S>>(
        &mut self,
        viewport: Viewport,
        colors: CT,
        depth: DT,
        stencil: ST,
    ) -> Option<ScopedPainter<C, D, S>> {
        // Convert the attachments to their untyped values
        let untyped_color = colors.untyped();
        let untyped_depth = depth.untyped();
        let untyped_stencil = stencil.untyped();

        // Check for any changes, and update the internal framebuffer if needed
        // We don't have to bother about us checking the discriminent of the enum since we already know they are Some using type checks (TODO: Explain this a better)
        // TODO: Delete duplicate OpenGL code
        // Update the color attachments and the draw buffers
        if untyped_color != self.untyped_color_attachments {
            self.untyped_color_attachments = untyped_color.clone();
            let mut offset = 0;
            for attachment in untyped_color.unwrap() {
                match attachment {
                    UntypedAttachment::TextureLevel {
                        texture_name,
                        level,
                        untyped: _,
                    } => unsafe {
                        gl::NamedFramebufferTexture(
                            self.name,
                            gl::COLOR_ATTACHMENT0 + offset,
                            texture_name,
                            level as i32,
                        );
                    },
                }
                offset += 1;
            }

            unsafe {
                let draw = (0..offset)
                    .into_iter()
                    .map(|offset| gl::COLOR_ATTACHMENT0 + offset)
                    .collect_vec();
                gl::NamedFramebufferDrawBuffers(
                    self.name,
                    draw.len() as i32,
                    draw.as_ptr() as *const u32,
                );
            }
        }

        // Update the depth attachment
        if untyped_depth != self.untyped_depth_attachment {
            self.untyped_depth_attachment = untyped_depth;
            let depth = untyped_depth.unwrap();
            match depth {
                UntypedAttachment::TextureLevel {
                    texture_name,
                    level,
                    untyped: _,
                } => unsafe {
                    gl::NamedFramebufferTexture(
                        self.name,
                        gl::DEPTH_ATTACHMENT,
                        texture_name,
                        level as i32,
                    );
                },
            }
        }

        // Update the stencil attachment
        if untyped_stencil != self.untyped_stencil_attachment {
            self.untyped_stencil_attachment = untyped_stencil;
            let stencil = untyped_stencil.unwrap();
            match stencil {
                UntypedAttachment::TextureLevel {
                    texture_name,
                    level,
                    untyped: _,
                } => unsafe {
                    gl::NamedFramebufferTexture(
                        self.name,
                        gl::STENCIL_ATTACHMENT,
                        texture_name,
                        level as i32,
                    );
                },
            }
        }

        Some(ScopedPainter {
            painter: self,
            viewport,
        })
    }
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
