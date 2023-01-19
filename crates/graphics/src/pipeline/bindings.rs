use crate::GraphicsPipeline;

// This is a wrapper that allows the user to send data to GPU shaders
// in a clean and safe fashion
pub struct Bindings {
}

impl Bindings {
    // Create some bindings for a specific type of graphics pipeline
    pub(crate) unsafe fn from_raw_parts(graphics: &GraphicsPipeline) -> Self {
        todo!()
    }

    // Set a push constants block without checking safety
    pub unsafe fn set_push_constant_block_unchecked<T: PushContantBlock>(
        &mut self,
        name: &'static str,
        value: &T,
    ) {
    }

    // Set a push constants block and make sure it's valid
    pub fn set_push_constant_block<T: PushContantBlock>(
        &mut self,
        name: &'static str,
        value: &T
    ) -> Option<()> {
        unsafe {
            self.set_push_constant_block_unchecked(name, value);
        }
    
        None
    }
}

// This trait will store the layout definition of a specific struct and
// allow the user to make sure the given layout in Bindings matches up
// with the layout within the graphical pipeline (safety)
pub trait PushContantBlock {
}

impl<T> PushContantBlock for T {}