use smallvec::SmallVec;

use crate::{registry, Archetype, ComponentError, Mask, QueryCache, StorageVecPtr, BorrowedItem};
// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub(crate) trait QueryLayout<'a> {
    // The tuple that will contain the pointers types of the specific query items
    type PtrTuple: 'static;

    // The safe tuple that will be given to the user
    type SafeTuple: 'a;

    // Get the number of entities that validate this query layout
    fn entity_len(archetype: &Archetype) -> usize {
        archetype.entities.len()
    }

    // Get the ptr tuple chunks from the cache
    fn get_filtered_chunks(cache: &QueryCache) -> Vec<(Self::PtrTuple, usize)>;

    // Get the combined mask of the query layout.
    fn layout_mask() -> Result<Mask, ComponentError>;
}

impl<'a, A: BorrowedItem<'a>> QueryLayout<'a> for A {
    type PtrTuple = *mut A::Component;
    type SafeTuple = A::Borrowed;
    
    fn layout_mask() -> Result<Mask, ComponentError> {
        registry::mask::<A::Component>()
    }

    fn get_filtered_chunks(cache: &QueryCache) -> Vec<(Self::PtrTuple, usize)> {
        let row = cache.get_row::<A::Component>();
        todo!()
    }
}
