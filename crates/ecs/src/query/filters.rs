use crate::{ComponentStateRow, Component};

// Query filter that will block certain bundles from being iterated
pub struct QueryFilter {
    condition: ComponentStateRow
}
