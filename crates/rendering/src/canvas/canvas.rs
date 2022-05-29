use crate::{canvas::rasterizer::Rasterizer, context::Context, shader::Shader};
use std::marker::PhantomData;

// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the device using the canvas() function
pub struct Canvas {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    _phantom: PhantomData<*const ()>,
}
impl Canvas {
    // Create a new canvas from the raw OpenGl ID of a framebuffer
    pub unsafe fn from_raw_parts(ctx: &mut Context, name: u32, size: vek::Extent2<u16>) -> Self {
        Self {
            name,
            size,
            _phantom: Default::default(),
        }
    }

    // Create a new canvas with a specific size (size must be valid)
    pub fn new(ctx: &mut Context, size: vek::Extent2<u16>) -> Self {
        // Validate size first
        assert_ne!(size, vek::Extent2::default(), "Size of canvas cannot be zero");

        // Create the raw OpenGL framebuffer
        let name = unsafe {
            let mut name = 0u32;
            gl::GenFramebuffers(1, &mut name);
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
        assert_ne!(new, vek::Extent2::default(), "Size of canvas cannot be zero");
        self.size = new;
    }

    // Get the current size of the canvas
    pub fn size(&self) -> vek::Extent2<u16> {
        self.size
    }

    // Bind the underlying framebuffer if it isn't bound already
    fn bind(&mut self, ctx: &mut Context) {
        // Make sure the framebuffer is bound, and that the viewport is valid
        ctx.bind(gl::FRAMEBUFFER, self.name, |name| unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, name);
            gl::Viewport(0, 0, self.size.w as i32, self.size.h as i32);
        });
    }

    // Clear the whole framebuffer using the proper flags
    pub fn clear(&mut self, ctx: &mut Context, color: Option<vek::Rgba<f32>>, depth: Option<f32>, stencil: Option<i32>) {
        // Accumulated bitwise flags that we will reset later
        let mut flags = 0u32;

        // Set the background color values
        if let Some(color) = color {
            unsafe {
                gl::ClearColor(color.r, color.g, color.g, color.a);
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
    // Get the canvas' rasterizer so we can draw stuff onto the canvas using a specific shader
    pub fn rasterizer<'canvas, 'shader, 'context>(&'canvas mut self, shader: &'shader mut Shader, ctx: &'context mut Context) -> Rasterizer<'canvas, 'shader, 'context> {
        Rasterizer {
            canvas: self,
            shader,
            context: ctx,
        }
    }
}
