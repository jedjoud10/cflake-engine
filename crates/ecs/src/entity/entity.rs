use crate::{EntityStateSet, Mask};
use slotmap::{new_key_type, SlotMap};
new_key_type! {
    pub struct Entity;
}

// Entity linking data that we will use to link entities to their specified components
#[derive(Default, Clone, Copy)]
pub struct EntityLinkings {
    // The component mask of this entity
    pub mask: Mask,

    // The index of the components in said archetype
    pub bundle: usize,
}
