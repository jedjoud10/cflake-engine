use std::intrinsics::transmute;

use crate::{Mask, EntityState, ArchetypalState, Component};

// Query filter that will block certain bundles
pub struct QueryFilter {
    entity: u8,
    components: Mask,
}

impl QueryFilter {
    // Entity state filter
    pub fn entity(state: ArchetypalState) -> QueryFilter {
        let state = unsafe { transmute::<ArchetypalState, u8>(state) };
        QueryFilter { entity: state, components: Mask::zero() }
    }

    // Component state filter
    // This will panic if the component is not registered
    pub fn changed<T: Component>() -> QueryFilter {

    }
}