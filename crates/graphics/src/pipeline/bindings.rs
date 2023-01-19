use vulkan::vk;
use crate::{GraphicsPipeline, PushConstantBlock, ModuleKind};

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
    pub unsafe fn set_push_constant_block_unchecked<T: PushableConstant>(
        &mut self,
        name: &'static str,
        flags: vk::ShaderStageFlags,
        value: &T,
    ) {
    }
    
    // Set a push constants block and make sure it's valid
    // This will atuomatically get the required shader stage flags for the block
    pub fn set_push_constant_block<T: PushableConstant>(
        &mut self,
        name: &'static str,
        value: &T
    ) -> Option<()> {
        None
    }
}

// This trait will store the layout definition of a specific struct and
// allow the user to make sure the given layout in Bindings matches up
// with the layout within the graphical pipeline (safety)
pub trait PushableConstant {
    fn layout() -> PushConstantBlock;
}