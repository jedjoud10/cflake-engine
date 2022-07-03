use crate::{context::Context, object::ToGlName, prelude::Uniforms, shader::Shader};
use std::marker::PhantomData;

use super::{RasterSettings, Rasterizer};

// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the device using the canvas() function
pub struct Canvas {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    // Unsend + Unsync
    _phantom: PhantomData<*const ()>,
}
impl Canvas {
    // Create a new canvas from the raw OpenGl ID of a framebuffer
    pub unsafe fn from_raw_parts(_ctx: &mut Context, name: u32, size: vek::Extent2<u16>) -> Self {
        Self {
            name,
            size,
            _phantom: Default::default(),
        }
    }

    // Create a new canvas with a specific size (size must be valid)
    pub fn new(_ctx: &mut Context, size: vek::Extent2<u16>) -> Self {
        // Validate size first
        assert_ne!(
            size,
            vek::Extent2::default(),
            "Size of canvas cannot be zero"
        );

        // Create the raw OpenGL framebuffer
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateFramebuffers(1, &mut name);
            name
        };

        // Then we can create the canvas object
        Self {
            name,
            size,
            _phantom: Default::default(),
        }
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
        // Accumulated bitwise flags that we will reset later
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
                flags |= gl::COLOR_BUFFER_BIT;
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
    pub fn rasterizer<'canvas, 'context, 'shader>(
        &'canvas mut self,
        ctx: &'context mut Context,
        shader: &'shader mut Shader,
        settings: RasterSettings,
    ) -> (Rasterizer<'canvas, 'context>, Uniforms<'shader>) {
        // Make sure the framebuffer is bound, and that the viewport is valid
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.name);
            gl::Viewport(0, 0, self.size.w as i32, self.size.h as i32);
        }

        // Bind the program, and set it's uniforms
        ctx.bind(gl::PROGRAM, shader.as_ref().name(), |obj| unsafe {
            gl::UseProgram(obj)
        });

        // Create the new rasterizer
        (
            Rasterizer::new(self, ctx, settings),
            Uniforms(shader.as_mut()),
        )
    }
}
