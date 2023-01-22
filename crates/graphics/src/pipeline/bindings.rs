use std::mem::{size_of, align_of};

use crate::{GraphicsPipeline, ModuleKind, PushConstantBlock};
use vulkan::vk;

// Smaller push constant range that can be updated
pub struct BlockSubRange {
    pub offset: u64,
    pub size: u64,
}

// Defines a push constant block's layout
pub struct BlockDefinition {
    pub ranges: Vec<BlockSubRange>,
    pub size: u32,
    pub alignment: u32,
}


// Since Vulkan is explicit, we must define the bindings config of each material before hand
#[derive(Default)]
pub struct BindingConfig {
    // Push constant blocks shit
    pub vertex_push_constant_block_definition: Option<BlockDefinition>,
    pub framgent_push_constant_block_definition: Option<BlockDefinition>,

    // Descriptor sets shit (bindless)
}

impl BindingConfig {
    // Create empty binding config that contains no shader variables
    pub fn empty() -> Self {
        Self::default()
    }

    // Create some new binding configs using block definitions and their corresponding shader
    pub fn from_block_definitions(defs: &[(ModuleKind, BlockDefinition)]) -> Self {
        todo!()
    }
}

// A push constant block's member (variable)
pub trait Member: Sized {
}

// A whole push constant block
pub trait Block: Sized {
    fn definition() -> BlockDefinition;
}

// This is a wrapper that allows the user to send data to GPU shaders
// in a clean and safe fashion
pub struct ActiveBindings {}

impl ActiveBindings {
    // Create some bindings for a specific type of graphics pipeline
    pub(crate) unsafe fn from_raw_parts(
        graphics: &GraphicsPipeline,
    ) -> Self {
        Self {}
    }

    // Update a sub-range of push constants within a push constant block without checking for safety
    pub unsafe fn set_unchecked<T: Member>(
        &mut self,
        block_name: &'static str,
        var_name: &'static str,
        value: &T
    ) {
        todo!()
    }
    
    // Update the whole push constant block without checking for safety
    pub unsafe fn set_block_unchecked<T: Block>(
        &mut self,
        block_name: &'static str,
        value: &T
    ) {
        todo!()
    }

    // Update a sub-range of push constants within a push constant block
    pub fn set<T: Member>(
        &mut self,
        block_name: &'static str,
        var_name: &'static str,
        value: &T
    ) -> Option<()> {
        todo!()
    }
    
    // Update the whole push constant block
    pub fn set_block<T: Block>(
        &mut self,
        block_name: &'static str,
        value: &T
    ) -> Option<()> {
        todo!()
    }
}