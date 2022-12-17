use ahash::{AHashSet, AHashMap};
use crate::{Scene, Entity, Component};

// A child component added onto entities that are linked to a parent entity
#[derive(Component)]
pub struct Child {
    pub(crate) parent: Entity,
    pub(crate) local_to_world: vek::Mat4<f32>,
    pub(crate) depth: usize,
}

impl Child {
    // Get the parent of this child
    pub fn parent(&self) -> Entity {
        self.parent
    }

    // Get the local to world matrix
    pub fn local_to_world(&self) -> vek::Mat4<f32> {
        self.local_to_world
    }
    
    // Get the world to local matrix
    pub fn world_to_local(&self) -> vek::Mat4<f32> {
        self.local_to_world.inverted()
    }
    
    // Get the depth of this child
    pub fn depth(&self) -> usize {
        self.depth
    }
}

// Parent component added onto entities that have multiple children
#[derive(Component)]
pub struct Parent {
    pub(crate) children: usize,
}

impl Parent {
    // Get the number of children that this parent has
}