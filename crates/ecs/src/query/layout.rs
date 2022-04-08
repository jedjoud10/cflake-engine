use crate::{Mask, QueryCache, Archetype};

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a> {
    // Get the combined mask of the query layout.
    fn mask() -> Mask;

    // Run the proper cache() function for each of the generic QueryItems
    fn cache_all(archetypes: &mut [Archetype], cache: &mut QueryCache);
}