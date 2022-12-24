use vulkan::vk;

use crate::{Compiled, ComputeModule, FragmentModule, VertexModule};

// This trait will be implemented for valid combinations of multiple unique modules
pub trait LinkedModules {
}

// Only used for graphic pipeline linked modules
pub trait GraphicsPipelineLinkedModules: LinkedModules {}

// Simple graphics pipeline linked modules
impl LinkedModules for (Compiled<VertexModule>, Compiled<FragmentModule>) {}
impl GraphicsPipelineLinkedModules for (Compiled<VertexModule>, Compiled<FragmentModule>) {}

// Compute shader module set
//impl LinkedModules for Compiled<ComputeModule> {}

