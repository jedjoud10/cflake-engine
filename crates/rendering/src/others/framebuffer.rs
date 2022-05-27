use std::marker::PhantomData;

use crate::context::Context;

// A framebuffer is what we usually draw into. This is only useful if we use deferred shading or if we render the scene to a single texture
pub struct Framebuffer {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    _phantom: PhantomData<*const ()>,
}

impl Framebuffer {
    // Create a new framebuffer with the proper settings 
    fn new(ctx: &mut Context, size: vek::Extent2<u16>) -> Self {
        Self {
            name: unsafe {
                let mut name = 0u32;
                gl::GenFramebuffers(1, &mut name);
                name
            },
            size,
            _phantom: Default::default(),
        }
    }

    // Get the framebuffer canvas (bind the current framebuffer to the context)
    pub fn canvas_mut<'a>(&'a mut self, ctx: &mut Context) -> Canvas<'a> {
        // Make sure the framebuffer is bound, and that the viewport is valid
        ctx.bind(gl::VIEWPORT, self.name, |target, name| unsafe {
            gl::BindFramebuffer(target, name);
            gl::Viewport(0, 0, self.size.w as i32, self.size.h as i32);
        });        
        
        Canvas {
            name: self.name,
            size: self.size,
            phantom_: Default::default(),
        }
    }

    // Resize the framebuffer (PS: This clears the framebuffer as well)
    pub fn resize(&mut self, ctx: &mut Context, size: vek::Extent2<u16>) {
        self.size = size;
    }

    
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.name);
        }
    }
}

// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the device using the canvas() function
pub struct Canvas<'a> {
    name: u32,
    size: vek::Extent2<u16>,
    phantom_: PhantomData<&'a ()>,
}

impl<'a> Canvas<'a> {
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
}