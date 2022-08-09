use super::{CanvasLayout, RasterSettings, Rasterizer};
use crate::{
    context::Context,
    object::ToGlName,
    prelude::{Depth, Ranged, Texel, TexelFormat, Uniforms, RGBA},
    shader::Shader,
};
use std::marker::PhantomData;
use world::{Handle, Storage, UntypedHandle};
// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the window using the canvas() function
pub struct Canvas<L: CanvasLayout> {
    name: u32,
    size: vek::Extent2<u16>,
    attachments: L,
    _phantom: PhantomData<*const ()>,
}

impl<L: CanvasLayout> Canvas<L> {
    // Create a new canvas from the raw OpenGl ID of a framebuffer
    // This assumes that the framebuffer was already initialized somewhere else
    pub unsafe fn from_raw_parts(
        _ctx: &mut Context,
        name: u32,
        size: vek::Extent2<u16>,
        attachments: L,
    ) -> Self {
        assert_ne!(
            size,
            vek::Extent2::default(),
            "Size of canvas cannot be zero"
        );

        Self {
            name,
            size,
            attachments,
            _phantom: Default::default(),
        }
    }

    // Create a new canvas with a specific size (size must be valid)
    pub fn new(_ctx: &mut Context, size: vek::Extent2<u16>, attachments: L) -> Option<Self> {
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            gl::BindFramebuffer(gl::FRAMEBUFFER, name);
            name
        };

        let mut draw_buffers = 0;
        let mut depth_enabled = false;
        let mut stencil_enabled = false;

        for canvas_attachment in attachments.iter() {
            let attachment = match canvas_attachment.format {
                TexelFormat::Color | TexelFormat::GammaCorrectedColor => {
                    let attachment = gl::COLOR_ATTACHMENT0 + draw_buffers;
                    draw_buffers += 1;
                    attachment
                }
                TexelFormat::Depth => {
                    if !depth_enabled {
                        depth_enabled = true;
                        gl::DEPTH_ATTACHMENT
                    } else {
                        return None;
                    }
                }
                TexelFormat::Stencil => {
                    if !stencil_enabled {
                        stencil_enabled = true;
                        gl::STENCIL_ATTACHMENT
                    } else {
                        return None;
                    }
                }
            };

            unsafe {
                gl::NamedFramebufferTexture(name, attachment, canvas_attachment.name, 0);
            }
        }

        unsafe {
            let vec = (0..draw_buffers)
                .map(|i| gl::COLOR_ATTACHMENT0 + i)
                .collect::<Vec<u32>>();
            gl::NamedFramebufferDrawBuffers(name, draw_buffers as i32, vec.as_ptr());
        }

        unsafe {
            let state = gl::CheckNamedFramebufferStatus(name, gl::FRAMEBUFFER);
            if state != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer initialization error {state}");
            }
        }

        Some(unsafe { Self::from_raw_parts(_ctx, name, size, attachments) })
    }

    // Resize the canvas to a new size
    pub fn resize(&mut self, new: vek::Extent2<u16>) {
        assert_ne!(
            new,
            vek::Extent2::default(),
            "Size of canvas cannot be zero"
        );
        self.size = new;
    }

    // Get the current size of the canvas
    pub fn size(&self) -> vek::Extent2<u16> {
        self.size
    }

    // Get the internal attachments (None if it is the default main canvas)
    pub fn attachments(&self) -> Option<L> {
        (self.name != 0).then(|| self.attachments)
    }

    // Clear the whole framebuffer using the proper flags
    // This will only clear the framebuffer's draw buffers if they are using floating point colors
    pub fn clear(
        &mut self,
        color: Option<vek::Rgb<f32>>,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        let mut flags = 0u32;

        // Set the background color values
        if let Some(color) = color {
            unsafe {
                gl::ClearColor(color.r, color.g, color.b, 1.0);
                flags |= gl::COLOR_BUFFER_BIT
            }
        }

        // Set the background depth values
        if let Some(depth) = depth {
            unsafe {
                gl::ClearDepth(depth as f64);
                flags |= gl::DEPTH_BUFFER_BIT;
            }
        }

        // Set the background stencil values
        if let Some(stencil) = stencil {
            unsafe {
                gl::ClearStencil(stencil);
                flags |= gl::STENCIL_BUFFER_BIT;
            }
        }

        // Clear the whole canvas using the proper bitwise flags
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.name);
            gl::Clear(flags);
        }
    }

    // Create a new canvas rasterizer that we can use to draw some 3D or 2D objects
    pub fn rasterizer<'shader: 'uniforms, 'canvas, 'context, 'uniforms>(
        &'canvas mut self,
        ctx: &'context mut Context,
        shader: &'shader mut Shader,
        settings: RasterSettings,
    ) -> (Rasterizer<'canvas, 'context, L>, Uniforms<'uniforms>) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.name);
            gl::Viewport(0, 0, self.size.w as i32, self.size.h as i32);
        }

        // Create the new rasterizer and it's corresponding uniforms
        let uniforms = Uniforms::new(shader.as_mut(), ctx);
        let rasterizer = Rasterizer::new(self, ctx, settings);
        (rasterizer, uniforms)
    }
}
