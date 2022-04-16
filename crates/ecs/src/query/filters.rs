use crate::{Component, ComponentStateRow};

// Input data given to the filters
pub struct FilterInput {
    bundle: usize,
    states: ComponentStateRow,
}

// Query filter that will block certain bundles from being iterated
pub struct QueryFilter {
    filter: fn(FilterInput) -> bool,
}

// The specified component needs to be added onto the entity in the current frame for this to pass
pub fn added<T: Component>(_i: FilterInput) -> bool {
    // Check the "add" state
    todo!()
}

// The component needs to be mutated for this filter to pass
pub fn mutated<T: Component>(_i: FilterInput) -> bool {
    todo!()
}
