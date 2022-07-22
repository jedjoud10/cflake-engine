use serde::__private::de;
use world::{UntypedHandle, Handle};

use crate::{context::Context, object::ToGlName, prelude::{Uniforms, Texture2D, Texel, TexelFormat, RenderTextureTuple}, shader::Shader};
use std::marker::PhantomData;

use super::{RasterSettings, Rasterizer};

// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the window using the canvas() function
pub struct Canvas {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    // Color attachements and depth/stencil attachements
    attachments: Vec<UntypedHandle>,

    // Unsend + Unsync
    _phantom: PhantomData<*const ()>,
}
impl Canvas {
    // Create a new canvas from the raw OpenGl ID of a framebuffer
    pub unsafe fn from_raw_parts(_ctx: &mut Context, name: u32, size: vek::Extent2<u16>) -> Self {
        assert_ne!(
            size,
            vek::Extent2::default(),
            "Size of canvas cannot be zero"
        );

        Self {
            name,
            size,
            attachments: Default::default(),
            _phantom: Default::default(),
        }
    }

    // Create a new canvas with a specific size (size must be valid)
    pub fn new(_ctx: &mut Context, size: vek::Extent2<u16>, targets: Vec<&dyn RenderTextureTuple>) -> Option<Self> {
        /*
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            name
        };

        let mut draw_buffers = 0;
        let mut depth_enabled = false;
        let mut stencil_enabled = false; 

        for target in targets {
            match target.texel_format() {
                TexelFormat::Color => draw_buffers += 1,
                TexelFormat::Depth => if !depth_enabled {
                    depth_enabled = true;
                } else {
                    return None;
                },
                TexelFormat::Stencil => if !stencil_enabled {
                    stencil_enabled = true
                } else {
                    return None;
                },
            }

            if target.size() != size {
                return None;
            }

            unsafe {
                gl::NamedFramebufferTexture(name, gl::COLOR_ATTACHMENT0 + draw_buffers as u32, target.name(), 0);
            }
        }

        unsafe {
            let vec = (0..draw_buffers).map(|i| gl::COLOR_ATTACHMENT0 + i).collect::<Vec<u32>>();
            gl::NamedFramebufferDrawBuffers(name, draw_buffers as i32, vec.as_ptr());
        }        

        unsafe {
            let state = gl::CheckNamedFramebufferStatus(name, gl::FRAMEBUFFER);
            if state != gl::FRAMEBUFFER_COMPLETE {
                panic!();
            }
        }    

        Some(unsafe { Self::from_raw_parts(_ctx, name, size) })
        */
        todo!()
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
