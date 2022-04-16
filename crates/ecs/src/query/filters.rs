// Query filter input that will be passed to the filtering function
pub struct QueryFilterInput {
    mutated: u64,
}

// Query filter that will block certain bundles from being iterated through
pub struct QueryFilter {
    // Filter
    filter: fn(QueryFilterInput),
}

impl QueryFilter {
    // Check if a specific bundle can pass the filter
    pub fn is_valid(&self, bundle: Bundle) -> bool {

    }
}
