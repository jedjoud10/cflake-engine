use crate::{registry, Archetype, ComponentError, Mask, QueryCache, QueryItem};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a> {
    // The tuple that will be given to the user
    type Tuple;

    // Get the combined mask of the query layout.
    fn layout_mask() -> Result<Mask, ComponentError>;

    // Run the cache() function for each of the generic QueryItems
    fn cache_items(archetype: &Archetype, cache: &mut QueryCache);
}

impl<A: QueryItem> QueryLayout<'_> for A {
    type Tuple = A::Item;
    
    fn layout_mask() -> Result<Mask, ComponentError> {
        A::item_mask()
    }

    fn cache_items(archetype: &Archetype, cache: &mut QueryCache) {
        A::cache(archetype, cache)
    }
}
