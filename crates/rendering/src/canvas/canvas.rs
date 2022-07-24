use serde::__private::de;
use world::{UntypedHandle, Handle, Storage};
use crate::{context::Context, object::ToGlName, prelude::{Uniforms, Texture2D, Texel, TexelFormat, RenderTarget2D, Depth, Ranged, RGBA}, shader::Shader};
use std::marker::PhantomData;
use super::{RasterSettings, Rasterizer};

// Framebuffer attachements
pub type ColorAttachment = RenderTarget2D<RGBA<Ranged<u8>>>;
pub type DepthAttachment = RenderTarget2D<Depth<Ranged<u32>>>;

// A canvas attachment is something that we can link to a canvas to be able to draw onto it
// I only implemented canvas attachments for RenderTarget2D, though that might change later
pub struct CanvasAttachment {
    handle: UntypedHandle,
    name: u32,
    format: TexelFormat,
}

// Trait that implements a valid framebuffer canvas attachments
// That that can be converted to a valid canvas attachments
pub trait ToCanvasAttachment {
    fn into(&self) -> CanvasAttachment;
}

impl<T: Texel> ToCanvasAttachment for (&'_ Storage<RenderTarget2D<T>>, Handle<RenderTarget2D<T>>) {
    fn into(&self) -> CanvasAttachment {
        let (storage, handle) = self;
        let tex = &storage[handle];
        CanvasAttachment { handle: handle.untyped().clone(), name: tex.name(), format: T::ENUM_FORMAT }
    }
}

// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the window using the canvas() function
pub struct Canvas {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    // Color attachements and depth/stencil attachements
    attachments: Vec<CanvasAttachment>,

    // Unsend + Unsync
    _phantom: PhantomData<*const ()>,
}

impl Canvas {
    // Create a new canvas from the raw OpenGl ID of a framebuffer
    // This assumes that the framebuffer was already initialized somewhere else
    pub unsafe fn from_raw_parts(_ctx: &mut Context, name: u32, size: vek::Extent2<u16>, attachments: Vec<CanvasAttachment>) -> Self {
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
    pub fn new(_ctx: &mut Context, size: vek::Extent2<u16>, targets: Vec<&dyn ToCanvasAttachment>) -> Option<Self> {
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            gl::BindFramebuffer(gl::FRAMEBUFFER, name);
            name
        };

        let attachments = targets.into_iter().map(|a| ToCanvasAttachment::into(a)).collect::<Vec<_>>();
        let mut draw_buffers = 0;
        let mut depth_enabled = false;
        let mut stencil_enabled = false; 

        for canvas_attachment in attachments.iter() {
            let attachment = match canvas_attachment.format {
                TexelFormat::Color => { draw_buffers += 1; gl::COLOR_ATTACHMENT0 + draw_buffers },
                TexelFormat::Depth => if !depth_enabled {
                    depth_enabled = true;
                    gl::DEPTH_ATTACHMENT
                } else {
                    return None;
                },
                TexelFormat::Stencil => if !stencil_enabled {
                    stencil_enabled = true;
                    gl::STENCIL_ATTACHMENT
                } else {
                    return None;
                },
            };

            unsafe {
                gl::NamedFramebufferTexture(name, attachment, canvas_attachment.name, 0);
            }
        }

        unsafe {
            let vec = (0..draw_buffers).map(|i| gl::COLOR_ATTACHMENT0 + i).collect::<Vec<u32>>();
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

    // Clear the whole framebuffer using the proper flags
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
            gl::Clear(flags);
        }
    }

    // Create a new canvas rasterizer that we can use to draw some 3D or 2D objects
    pub fn rasterizer<'shader: 'uniforms, 'canvas, 'context, 'uniforms>(
        &'canvas mut self,
        ctx: &'context mut Context,
        shader: &'shader mut Shader,
        settings: RasterSettings,
    ) -> (Rasterizer<'canvas, 'context>, Uniforms<'uniforms>) {
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
