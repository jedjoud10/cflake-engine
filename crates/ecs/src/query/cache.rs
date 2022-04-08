use std::{any::Any, collections::{HashMap, hash_map::Entry}};
use crate::{Mask, MaskHasher, QueryLayout};

// Query cache that contains multiple boxed vectors of multiple layouts
pub struct QueryCache {
    // Boxed query vectors (real type is Vec<T> where T: Layout)
    // Vec<Vec<T> where T: Layout>
    vecs: HashMap<Mask, Box<dyn Any>, MaskHasher>,
}

impl QueryCache {
    // Make sure a boxed vector (with specified capacity) already exists
    pub fn insert_with_capacity<'a, Layout: QueryLayout<'a> + 'static>(&mut self, mask: Mask, capacity: usize) -> Option<()> {
        // Check first
        if self.vecs.contains_key(&mask) { return None; }

        // Vector init
        let vec = Vec::<Layout>::with_capacity(capacity);
        let boxed: Box::<dyn Any> = Box::new(vec);
        self.vecs.insert(mask, boxed);
        Some(())
    }
}
