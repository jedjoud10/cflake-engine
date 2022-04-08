use crate::{Archetype, QueryCache};

// Something that can be queried. This will be implement on Read<T> and Write<T> (where T is Component). This will also be implemented on Read<Entity>
pub trait QueryItem<'a> {
    // Add the necessary data into the query cache
    fn cache(archetype: &Archetype, cache: &mut QueryCache) {

    }
}
