use std::{mem::{size_of, align_of}, marker::PhantomData, any::{TypeId, Any}, cell::Cell};

use crate::{GraphicsPipeline, ModuleKind, PushConstantBlock, BlockVariable, VariableType, GpuPod};
use ahash::{AHashMap, AHashSet};
use crate::vulkan::{vk, Recorder};

// This contains the config for the multiple module binding configs in one structure
pub type BindingConfig = AHashMap<ModuleKind, ModuleBindingConfig>;

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
        self.push_constant = Some((B::definition(), TypeId::of::<B>()));
        self
    }
}

// A push constant block's member (variable)
pub trait Member: Sized {
    fn definition() -> BlockVariable;
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