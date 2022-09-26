use itertools::Itertools;

use super::{
    ColorAttachmentLayout, DepthAttachment, PainterColorLayout, PainterDepthTexel,
    PainterStencilTexel, StencilAttachment, UntypedAttachment, Target,
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
    pub fn scope<CT: ColorAttachmentLayout<C>>(
        &mut self,
        viewport: Viewport,
        colors: CT,
        depth: <Target<D>,
        stencil: Target<S>,
    ) -> Option<ScopedPainter<C, D, S>> {
        // Convert the attachments to their untyped values
        let untyped_color = colors.untyped();
        let untyped_depth = depth.untyped;
        let untyped_stencil = stencil.untyped;

        // Simple local struct to help us bind the attachments to the painter 
        struct Attachment {
            untyped: UntypedAttachment,
            code: u32,
        }

        // We will bind all the attachments later
        let mut attachments = Vec::<Attachment>::new();

        // Convert the untyped color attachments to the local struct
        if untyped_color != self.untyped_color_attachments {
            let untyped_color = untyped_color.unwrap();
            let mut offset = 0;
            attachments.extend(untyped_color.iter().map(|untyped| {
                let attachment = Attachment {
                    untyped: untyped.clone(),
                    code: gl::COLOR_ATTACHMENT0 + offset,
                };
                offset += 1;
                return attachment;
            }));
            self.untyped_color_attachments = Some(untyped_color);
        }

        // Convert the untyped depth attachment to the local struct
        if untyped_depth != self.untyped_depth_attachment {
            let untyped_depth = untyped_depth.unwrap();
            self.untyped_depth_attachment = Some(untyped_depth);
            attachments.push(Attachment {
                untyped: untyped_depth,
                code: gl::DEPTH_ATTACHMENT,
            });
        }

        // Convert the untyped stencil attachment to the local struct
        if untyped_stencil != self.untyped_stencil_attachment {
            let untyped_stencil = untyped_stencil.unwrap();
            self.untyped_stencil_attachment = Some(untyped_stencil);
            attachments.push(Attachment {
                untyped: untyped_stencil,
                code: gl::STENCIL_ATTACHMENT,
            });
        }

        // Bind the texture layers/levels to the proper attachments
        for attachment in attachments.iter() {
            match attachment.untyped {
                UntypedAttachment::TextureLevel {
                    texture_name,
                    level,
                    untyped: _,
                } => unsafe {
                    gl::NamedFramebufferTexture(
                        self.name,
                        attachment.code,
                        texture_name,
                        level as i32,
                    );
                },
                UntypedAttachment::TextureLevelLayer { texture_name, level, layer, untyped } => unsafe {
                    gl::NamedFramebufferTextureLayer(
                        self.name,
                        attachment.code,
                        texture_name,
                        level as i32,
                        layer as i32
                    );
                },
            }
        }

        // Check if we have any color attachment, and if we do, check how many of them we have bound to the FB
        let color_attachments_bound: Option<u32> = attachments.iter().fold(None, |current, item| {
            match item.code {
                gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT31 => match current {
                    Some(x) => Some(x + 1),
                    None => Some(1),
                }
                _ => None,
            }
        });

        // Apply the color draw buffers
        if let Some(count) = color_attachments_bound {
            let draw = (0..count)
                    .into_iter()
                    .map(|offset| gl::COLOR_ATTACHMENT0 + offset)
                    .collect_vec();

            unsafe {
                gl::NamedFramebufferDrawBuffers(
                    self.name,
                    draw.len() as i32,
                    draw.as_ptr() as *const u32,
                );
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
