use crate::{Component, Entity, Scene};
use ahash::{AHashMap, AHashSet};

// A child component added onto entities that are linked to a parent entity
#[derive(Component)]
pub struct Child {
    pub(crate) parent: Entity,
    pub(crate) depth: usize,
}

impl Child {
    // Get the parent of this child
    pub fn parent(&self) -> Entity {
        self.parent
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
    pub fn children(&self) -> usize {
        self.children
    }
}
