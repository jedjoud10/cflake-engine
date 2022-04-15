// Query filter input that will be passed to the filtering function
pub struct QueryFilterInput {}

// Query filter that will block certain bundles from being iterated through
pub struct QueryFilter {
    // Filter
    filter: fn(QueryFilterInput),
}

impl QueryFilter {}
