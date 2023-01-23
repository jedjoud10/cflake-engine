use std::{mem::{size_of, align_of}, marker::PhantomData, any::{TypeId, Any}, cell::Cell};

use crate::{GraphicsPipeline, ModuleKind, PushConstantBlock, BlockVariable, VariableType, GpuPod};
use ahash::{AHashMap, AHashSet};
use vulkan::{vk, Recorder};

// This contains the config for the multiple module binding configs in one structure
#[derive(Default)]
pub struct BindingConfig(pub(crate) AHashMap<ModuleKind, ModuleBindingConfig>);

impl BindingConfig {
    pub fn from_modules(slice: &[(ModuleKind, ModuleBindingConfig)]) -> Self {
        todo!()
    }
}

// Since Vulkan is explicit, we must define the bindings config of each material before hand
// This binding config is for a single module only
#[derive(Default)]
pub struct ModuleBindingConfig {
    // Push constant blocks shit
    pub push_constant: Option<(PushConstantBlock, TypeId)>,

    // Descriptor sets shit (bindless)
}

impl ModuleBindingConfig {
    // Enables the usage of a specific push constant block within the module bindings
    pub fn with_push_constant<B: Block>(mut self) -> Self {
        self.push_constant.insert((B::definition(), TypeId::of::<B>()));
        self
    }
}

// A push constant block's member (variable)
pub trait Member: Sized {
    fn size() -> u32;
    fn var_type() -> VariableType;
}

// Trait implemented for structs that have a #[derive(PushConstantBlock)]
// and the appropriate attributes on each of their fields
pub trait Block: Sized + 'static {
    // Internal RAW representation of the push constant
    // Have to do this cause of alignment and cause bool is actually a 32 bit int (in SPIRV)
    type Internal: GpuPod + Sized + 'static;
    fn definition() -> PushConstantBlock;
    fn serialize(&self) -> Self::Internal;
}

// This is a wrapper that allows the user to send data to GPU shaders
// in a clean and safe fashion
// TODO: Figure out how to handle bindless textures and buffers?
pub struct Bindings<'a> {
    config: &'a BindingConfig,
    layout: vk::PipelineLayout,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Bindings< 'a> {
    // Create some bindings for a specific type of graphics pipeline
    pub(crate) unsafe fn from_raw_parts(
        graphics: &'a GraphicsPipeline,
    ) -> Self {
        Self {
            config: graphics.binding_config(),
            layout: graphics.layout(),
            _phantom: PhantomData,
        }
    }

    // Update the whole push constant block
    pub fn set_block<B: Block>(
        &mut self,
        block_name: &'static str,
        value: &B
    ) -> Option<()> {
        // We iterate because I want the user to be able to call "set_block" once for all recurrent block defs in each module
        for (kind, module_binding_config) in &self.config.0 {
            if let Some((block, _type)) = &module_binding_config.push_constant {
                // Check if the block is the same as defined in the config
                // (which is also the same block as defined in the shader through reflection)
                if TypeId::of::<B>() == *_type {
                    // Set the block using cmdPushConstants
                    let internal = value.serialize();
                    let boxed: Box<dyn Any> = Box::new(internal);
                
                    //self.push_constants.insert(*kind, (Cell::new(true), boxed));
                } else {
                    // Block definition mismatch
                    return None;
                }
            } else {
                // Block not defined
                return None;
            }
        }

        Some(())
    }

    // Update a sub-range of push constants within a push constant block
    // This assumes that the variable is set as dynamic within the defintion of Block "B"
    pub fn set<M: Member, B: Block>(
        &mut self,
        block_name: &'static str,
        var_name: &'static str,
        value: &M
    ) -> Option<()> {
        todo!()
    }
}