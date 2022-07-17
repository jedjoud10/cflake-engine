use crate::Mask;
use slotmap::new_key_type;
new_key_type! {
    pub struct Entity;
}

// Entity linking data that we will use to link entities to their specified components
#[derive(Clone, Copy)]
pub struct EntityLinkings {
    pub(crate) mask: Mask,
    pub(crate) index: usize,
}

impl EntityLinkings {
    // Get the mask of the entity (the mask of it's current archetype)
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the index of the entity in it's current archetype
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Default for EntityLinkings {
    fn default() -> Self {
        Self {
            mask: Mask::zero(),
            index: Default::default(),
        }
    }
}
