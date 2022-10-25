use itertools::Itertools;

use super::{
    AsTarget, ColorTupleTargets, MaybeColorLayout, MaybeDepthTexel, MaybeStencilTexel, Target,
    UntypedAttachment,
};
use crate::{
    context::Context,
    display::{Display, Viewport},
    prelude::Texel,
};
use std::marker::PhantomData;

// A painter is a safe wrapper around an OpenGL framebuffer
// Painters only store the texel types that we shall use, but they do not store the attachments by themselves
pub struct Painter<C: MaybeColorLayout, D: MaybeDepthTexel, S: MaybeStencilTexel> {
    pub(super) name: u32,
    pub(crate) untyped_color_attachments: Option<Vec<UntypedAttachment>>,
    pub(crate) untyped_depth_attachment: Option<UntypedAttachment>,
    pub(crate) untyped_stencil_attachment: Option<UntypedAttachment>,
    bitmask: u32,
    _phantom: PhantomData<*const (C, D, S)>,
}

impl<C: MaybeColorLayout, D: MaybeDepthTexel, S: MaybeStencilTexel> Painter<C, D, S> {
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
            bitmask: 0,
            _phantom: PhantomData,
        }
    }

    // Convert a bit index to a OpenGL attachment code
    // 0-29 -> color
    // 30 -> depth
    // 31 -> stencil
    pub fn convert_signed_bit_to_code(location: u32) -> u32 {
        if (0..=29).contains(&location) {
            gl::COLOR_ATTACHMENT0 + location
        } else if location == 30 {
            gl::DEPTH_ATTACHMENT
        } else if location == 31 {
            gl::STENCIL_ATTACHMENT
        } else {
            panic!()
        }
    }

    // Given the three untyped attachment vectors/options from the painter, we must select a single one based on the given bitmask location
    fn select_untyped_attachment(&self, location: u32) -> Option<UntypedAttachment> {
        match location {
            0..=29 => self
                .untyped_color_attachments
                .as_ref()
                .map(|vec| vec[location as usize]),
            30 => self.untyped_depth_attachment,
            31 => self.untyped_stencil_attachment,
            _ => panic!(),
        }
    }

    // Use the painter to give us a scoped painter that has proper targets
    pub fn scope<CTS: ColorTupleTargets<C>, DT: AsTarget<D>, ST: AsTarget<S>>(
        &mut self,
        viewport: Viewport,
        color: CTS,
        depth: DT,
        stencil: ST,
    ) -> Option<ScopedPainter<C, D, S>> {
        // Convert the color attachments to their untyped attachments form
        let untyped_color = color.untyped_targets().map(|targets| {
            targets
                .into_iter()
                .map(|target| target.untyped)
                .collect_vec()
        });

        // Don't do anything to the depth and stencil
        let untyped_depth = depth.as_target().map(|target| target.untyped);
        let untyped_stencil = stencil.as_target().map(|target| target.untyped);

        // Check if we modified the painter in any way
        let changed = untyped_color != self.untyped_color_attachments
            || untyped_depth != self.untyped_depth_attachment
            || untyped_stencil != self.untyped_stencil_attachment;

        // Simple struct to help us bind the attachments to the painter
        struct Attachment {
            pub(crate) untyped: UntypedAttachment,
            pub(crate) code: u32,
        }

        // We will bind all the attachments later
        let mut attachments = Vec::<Attachment>::new();

        // Convert the untyped color attachments to the local struct
        if untyped_color != self.untyped_color_attachments {
            self.bitmask &= 0xC0000000;
            let untyped_color = untyped_color.unwrap();
            let mut offset = 0;
            attachments.extend(untyped_color.iter().map(|untyped| {
                let attachment = Attachment {
                    untyped: untyped.clone(),
                    code: gl::COLOR_ATTACHMENT0 + offset,
                };

                self.bitmask |= 1 << offset;
                offset += 1;
                return attachment;
            }));
            self.untyped_color_attachments = Some(untyped_color);
        }

        // Convert the untyped depth attachment to the local struct
        if untyped_depth != self.untyped_depth_attachment {
            self.bitmask &= !(1 << 30);
            let untyped_depth = untyped_depth.unwrap();
            self.untyped_depth_attachment = Some(untyped_depth);
            attachments.push(Attachment {
                untyped: untyped_depth,
                code: gl::DEPTH_ATTACHMENT,
            });
            self.bitmask |= 1 << 30;
        }

        // Convert the untyped stencil attachment to the local struct
        if untyped_stencil != self.untyped_stencil_attachment {
            self.bitmask &= !(1 << 31);
            let untyped_stencil = untyped_stencil.unwrap();
            self.untyped_stencil_attachment = Some(untyped_stencil);
            attachments.push(Attachment {
                untyped: untyped_stencil,
                code: gl::STENCIL_ATTACHMENT,
            });
            self.bitmask |= 1 << 31;
        }

        // Bind the texture layers/levels to the proper attachments
        for attachment in attachments.iter() {
            match attachment.untyped {
                UntypedAttachment::TextureLevel {
                    texture: texture_name,
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
                UntypedAttachment::TextureLevelLayer {
                    texture: texture_name,
                    level,
                    layer,
                    untyped: _,
                } => unsafe {
                    gl::NamedFramebufferTextureLayer(
                        self.name,
                        attachment.code,
                        texture_name,
                        level as i32,
                        layer as i32,
                    );
                },
            }
        }

        // Check if we have any color attachment, and if we do, check how many of them we have bound to the FB
        let color_attachments_bound: Option<u32> =
            attachments
                .iter()
                .fold(None, |current, item| match item.code {
                    gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT29 => match current {
                        Some(x) => Some(x + 1),
                        None => Some(1),
                    },
                    _ => None,
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

        // If the writing bitmask is different, we must check for framebuffer validity
        if changed {
            unsafe {
                let status = gl::CheckNamedFramebufferStatus(self.name, gl::FRAMEBUFFER);
                if status != gl::FRAMEBUFFER_COMPLETE {
                    println!("{:b}", self.bitmask);
                    //panic!("Framebuffer status error: {:?}", status);
                }
            }
        }

        Some(ScopedPainter {
            writing_mask: self.bitmask,
            painter: self,
            viewport,
        })
    }
}

// A scoped painter is what we must use to be able to use the Display's trait functionality
pub struct ScopedPainter<'a, C: MaybeColorLayout, D: MaybeDepthTexel, S: MaybeStencilTexel> {
    painter: &'a mut Painter<C, D, S>,
    writing_mask: u32,
    viewport: Viewport,
}

impl<C: MaybeColorLayout, D: MaybeDepthTexel, S: MaybeStencilTexel> Display
    for ScopedPainter<'_, C, D, S>
{
    fn viewport(&self) -> crate::display::Viewport {
        self.viewport
    }

    fn name(&self) -> u32 {
        self.painter.name
    }
}
