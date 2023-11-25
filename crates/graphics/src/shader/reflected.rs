use std::{hash::Hash, num::NonZeroU32, sync::Arc};
use super::ModuleVisibility;

// Visiblity for the set push constants bitset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PushConstantLayout {
    Single(NonZeroU32, ModuleVisibility),
    SplitVertexFragment {
        vertex: NonZeroU32,
        fragment: NonZeroU32,
    },
}

impl PushConstantLayout {
    // Create a push constant layout for a single module or SharedVG modules
    pub fn single(size: usize, visibility: ModuleVisibility) -> Option<Self> {
        let size = NonZeroU32::new(size as u32)?;
        Some(Self::Single(size, visibility))
    }

    // Create a push constant layout for a vertex module
    pub fn vertex(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Vertex)
    }

    // Create a push constant layout for a fragment module
    pub fn fragment(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Fragment)
    }

    // Create a push constant layout for a compute module
    pub fn compute(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Compute)
    }

    // Create a push constant layout for split vertex / fragment modules
    pub fn split(vertex: usize, fragment: usize) -> Option<Self> {
        let vertex = NonZeroU32::new(vertex as u32)?;
        let fragment = NonZeroU32::new(fragment as u32)?;
        Some(Self::SplitVertexFragment { vertex, fragment })
    }

    // Convert this push constant layout to it's ModuleVisibility
    pub fn visibility(&self) -> ModuleVisibility {
        match self {
            PushConstantLayout::Single(_, visibility) => *visibility,
            PushConstantLayout::SplitVertexFragment { .. } => ModuleVisibility::VertexFragment,
        }
    }

    // Get the MAX size required to set the push constant bytes
    pub fn size(&self) -> NonZeroU32 {
        match self {
            PushConstantLayout::Single(x, _) => *x,
            PushConstantLayout::SplitVertexFragment { vertex, fragment } => {
                NonZeroU32::new(vertex.get() + fragment.get()).unwrap()
            }
        }
    }
}