use std::ops::{BitOr, BitAnd};
use crate::{EntityState, Mask, Component, registry};

// Only return the bunldes where the component T was mutated
pub fn changed<T: Component>() -> QueryFilter {
    QueryFilter {
        changed_mask: registry::mask::<T>().unwrap(),
        entity: EntityState::None,
    }
}

// Only return the bundles were the entity state is equal to the given one
pub fn state(state: EntityState) -> QueryFilter {
    QueryFilter {
        changed_mask: todo!(),
        entity: todo!(),
    }
}