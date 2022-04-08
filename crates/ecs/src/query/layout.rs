use crate::{registry, Archetype, ComponentError, Mask, QueryCache, QueryItem};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a> {
    // The tuple that will contain the pointers types of the specific query items
    type PtrTuple: 'static;

    // The safe tuple that will be given to the user
    type SafeTuple: 'a;

    // Get the number of entities that validate this query layout
    fn entity_len(archetype: &Archetype) -> usize {
        archetype.entities().len()
    }

    // Get the combined mask of the query layout.
    fn layout_mask() -> Result<Mask, ComponentError>;

    // Run the cache() function for each of the generic QueryItems
    fn cache_items(archetype: &Archetype, cache: &mut QueryCache);
}

impl<'a, A: QueryItem<'a>> QueryLayout<'a> for A {
    type PtrTuple = *mut A::Item;
    type SafeTuple = A::BorrowedItem;
    
    fn layout_mask() -> Result<Mask, ComponentError> {
        A::item_mask()
    }

    fn cache_items(archetype: &Archetype, cache: &mut QueryCache) {
        // Make sure the query layout is cached
        cache.insert_with_capacity::<Self>(Self::entity_len(archetype));
        let chunk = cache.get_mut::<Self>().unwrap();

        // Insert the component pointers into the cache chunk
        A::cache(archetype, chunk)
    }
}
