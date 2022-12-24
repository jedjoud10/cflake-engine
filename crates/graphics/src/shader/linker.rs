use crate::{Compiled, VertexModule, FragmentModule, ComputeModule};

// This trait will be implemented for valid combinations of multiple unique modules
pub trait StageSet {
}

// Basic graphics stage set
impl StageSet for (Compiled<VertexModule>, Compiled<FragmentModule>) {

}

// Compute shader stage set
impl StageSet for Compiled<ComputeModule> {

}