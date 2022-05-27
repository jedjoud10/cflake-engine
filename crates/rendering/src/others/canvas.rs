
// A framebuffer canvas is an abstraction that we can use to modify the internal colors of the framebuffers
// We can access the main default canvas from the device using the canvas() function
pub struct Canvas<'a>(&'a mut Framebuffer);

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

    // Get the canvas' rasterizer so we can draw stuff onto the canvas
    pub fn rasterizer(&'a mut self) -> Rasterizer<'a> {
        Rasterizer(self.0)
    }
}
