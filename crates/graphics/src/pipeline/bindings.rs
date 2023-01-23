use std::mem::{size_of, align_of};

use crate::{GraphicsPipeline, ModuleKind, PushConstantBlock, PushConstantVariable};
use ahash::AHashMap;
use vulkan::vk;

// Since Vulkan is explicit, we must define the bindings config of each material before hand
#[derive(Default)]
pub struct BindingConfig {
    // Push constant blocks shit
    pub block_definitions: AHashMap<ModuleKind, PushConstantBlock>,

    // Descriptor sets shit (bindless)
}

impl BindingConfig {
    // Create empty binding config that contains no shader variables
    pub fn empty() -> Self {
        Self::default()
    }

    // Create some new binding configs using block definitions and their corresponding shader
    pub fn from_block_definitions(block_definitions: &[(ModuleKind, PushConstantBlock)]) -> Self {
        let block_definitions = block_definitions
            .into_iter()
            .cloned()
            .collect::<AHashMap<ModuleKind, PushConstantBlock>>();
        Self { block_definitions }
    }

    // Get the block definitions with their appropriate module kinds
    pub fn block_definitions(&self) -> &AHashMap<ModuleKind, PushConstantBlock> {
        &self.block_definitions
    }
}

// A push constant block's member (variable)
pub trait Member: Sized {
    fn definition() -> PushConstantVariable;
}

// Trait implemented for structs that have a #[derive(PushConstantBlock)]
// and the appropriate attributes on each of their fields
pub trait Block: Sized {
    fn definition() -> PushConstantBlock;
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