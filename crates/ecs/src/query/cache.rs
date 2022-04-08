use parking_lot::RwLock;
use smallvec::SmallVec;

use crate::{Mask, MaskHasher, QueryLayout};
use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap}, sync::atomic::AtomicPtr, ffi::c_void,
};

// Query cache that contains multiple boxed vectors of multiple layouts
pub struct QueryCache {
    // Boxed query vectors (real type is RwLock<Vec<T>> where T: Layout)
    // Vec<Vec<T> where T: Layout>
    vecs: HashMap<Mask, Box<dyn Any>, MaskHasher>,
}

impl QueryCache {
    // Make sure a boxed vector (with specified capacity) already exists
    pub fn insert_with_capacity<'a, Layout: QueryLayout<'a> + 'static>(&mut self, mask: Mask, capacity: usize) -> Option<()> {
        // Check first
        if self.vecs.contains_key(&mask) {
            return None;
        }

        // Vector init
        let vec = Vec::<Layout>::with_capacity(capacity);
        let boxed: Box<dyn Any> = Box::new(vec);
        self.vecs.insert(mask, boxed);
        Some(())
    }
    // Get a layout vector using it's layout type
    pub fn get_mut<'a, Layout: QueryLayout<'a>>(&mut self) -> Option<&mut Vec<Layout::Tuple>> {
        let boxed = self.vecs.get_mut(&Layout::layout_mask().ok()?)?;
        let vec = boxed.downcast_mut::<Vec<Layout>>()?;
        Some(vec)
    }
}
