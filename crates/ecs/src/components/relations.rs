use crate::{Scene, Entity, Component};

// This component will be added onto entities that will be linked to other entities in the scene hierarchy
#[derive(Component)]
pub struct Child {
    parent: Entity,
    depth: usize,
}

impl Child {
    // Create a new child entity by linking it to a parent
    pub fn new(parent: Entity) -> Self {
        Self {
            parent,
            depth: usize::MAX 
        }
    }

    // Get the parent of this child
    pub fn parent(&self) -> Entity {
        self.parent
    }

    // Get the depth of this child within the hierarchy
    pub fn depth(&self) -> &usize {
        &self.depth
    }
}