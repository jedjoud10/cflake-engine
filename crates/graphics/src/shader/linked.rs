use crate::{Compiled, ComputeModule, FragmentModule, VertexModule};

// This trait will be implemented for valid combinations of multiple unique modules
pub trait LinkedModules {}

// Basic graphics stage set
impl LinkedModules for (Compiled<VertexModule>, Compiled<FragmentModule>) {}

// Compute shader stage set
impl LinkedModules for Compiled<ComputeModule> {}
