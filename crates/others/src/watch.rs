use std::{sync::{atomic::{AtomicU16, Ordering::{Relaxed, self}, AtomicU64, AtomicUsize}, RwLock}, marker::PhantomData, collections::HashSet};
use ordered_vec::simple::OrderedVec;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

// A watchable trait. Implemented on structs that can be detected whenever they execute
pub trait Watchable<U> {
    fn is_valid(&self, context: &U) -> bool;
}

// A thread local watcher that can detect when certain "things" happen
pub struct Watcher<T, U> 
    where T: Watchable<U>
{
    phantom: PhantomData<*const U>,
    watchables: OrderedVec<(T, usize)>, // The watchable values that are waiting to become valid
    valids: HashSet<usize>, // The watchable values that have become valid
}

impl<T, U> Default for  Watcher<T, U>
    where T: Watchable<U>
{
    fn default() -> Self {
        Self { 
            phantom: PhantomData::default(),
            watchables: OrderedVec::default(),
            valids: HashSet::new()
        }
    }
}



impl<T, U> Watcher<T, U>
    where T: Watchable<U>
{
    // Add a watchable object to the watcher
    pub fn add(&mut self, watchable: T) {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        self.watchables.push_shove((watchable, id));
    }
    // Update the watcher, checking each value if it has become valid. If values did become valid, we must store their UID in self, since we will share self 
    // Update this at the end of each frame, after we do everything task related on the main thread
    pub fn update(&mut self, context: &U) {
        // Detect the values that became valid
        let validated = self.watchables.my_drain(|_, (val, id)| val.is_valid(context)).collect::<Vec<_>>();
        // And we will set the values that became valid
        self.valids = validated.iter().map(|(_, (val, idx))| idx).collect::<HashSet<usize>>();
    }
    // Check if a value became valid (This can be called on any thread, as long as we have a reference to self)
    pub fn has_become_valid(&self, watchable: &T) -> bool {
        self.valids.contains(&watchable.get_uid())
    }
}