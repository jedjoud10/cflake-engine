use std::{collections::HashMap, any::Any};

use crate::{Mask, MaskHasher};

// Query cache that contains multiple boxed vectors of multiple layouts
pub struct QueryCache {
    // Boxed query vectors (real type is Vec<T> where T: Layout)
    // Vec<Vec<T> where T: Layout>
    vecs: HashMap<Mask, Box<dyn Any>, MaskHasher>,
}

impl QueryCache {
    // Make sure a boxed vector that has a specific layout already exists
    pub fn insert<Layout: QueryLayout>(&mut self) -> &mut Vec<Layout> {

    }
}