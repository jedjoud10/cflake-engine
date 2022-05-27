use crate::context::Context;

// A framebuffer is what we usually draw into. When we draw something onto the screen, it first gets drawn into a framebuffer that then gets displayed
pub struct Framebuffer {
    // The raw framebuffer name (This can be 0 to depict the default framebuffer)
    name: u32,

    // The size of the framebuffer, in pixels
    size: vek::Extent2<u16>,

    // The currently bound draw attachements for this framebuffers
    draw: Option<Vec<u32>>,
}

impl Framebuffer {
    // Create a new framebuffer with the proper settings 
    fn new() -> Self {

    }

    // Resize the framebuffer (PS: This clears the framebuffer as well)
    pub fn resize(&mut self, ctx: &mut Context) {

    }

    // Clear the whole framebuffer using the proper flags
    pub fn clear(&mut self, ctx: &mut Context, color: Option<vek::Rgb<f32>>, depth: Option<f32>, stencil: Option<u32>) {


        // Accumulated bitwise flags that we will reset later
        
        if let Some(color) = color {
            // SEt the 
        }
    }
}