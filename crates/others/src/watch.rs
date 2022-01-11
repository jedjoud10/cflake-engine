use std::{sync::{atomic::{AtomicU16, Ordering::{Relaxed, self}, AtomicU64, AtomicUsize}, RwLock}, marker::PhantomData, collections::HashSet};
use ordered_vec::{simple::OrderedVec, shareable::ShareableOrderedVec};


// A watchable trait. Implemented on structs that can be detected whenever they execute
pub trait Watchable<U> {
    fn is_valid(&self, context: &U) -> bool;
}

// A watcher entry that can be stored inside the watcher. This also stores the watchable's UID
pub struct WatcherEntry<U, T: Watchable<U>> {
    phantom_: PhantomData<*const U>,
    phantom_2: PhantomData<*const T>,
    idx: usize,
}

// A command that we can use to insert a WatcherEntry into the Watcher
pub struct WatcherEntryInsert<U, T: Watchable<U>> {
    phantom_: PhantomData<*const U>,
    data: T,
    idx: usize,
}

impl<U, T: Watchable<U>> WatcherEntryInsert<U, T> {
    // Create a new watcher entry insert that we can send to the main thread
    pub fn new(data: T, idx: usize) -> Self {
        Self {
            phantom_: PhantomData::default(),
            data,
            idx,
        }
    }
}

// A thread local watcher that can detect when certain "things" happen
pub struct Watcher<T, U> 
    where T: Watchable<U>
{
    phantom_: PhantomData<*const U>,
    phantom_2: PhantomData<*const T>,
    watchables: ShareableOrderedVec<T>, // The watchable values that are waiting to become valid
    valids: HashSet<usize>, // The watchable values that have become valid
}

impl<T, U> Default for  Watcher<T, U>
    where T: Watchable<U>
{
    fn default() -> Self {
        Self { 
            phantom_: PhantomData::default(),
            phantom_2: PhantomData::default(),
            watchables: ShareableOrderedVec::default(),
            valids: HashSet::new()
        }
    }
}


fn extract<U, T: Watchable<U>>(insert: &WatcherEntryInsert<U, T>) -> WatcherEntry<U, T> {
    let idx = insert.idx;
    WatcherEntry {
        phantom_: PhantomData::default(),
        phantom_2: PhantomData::default(),
        idx,
    }
}


impl<T, U> Watcher<T, U>
    where T: Watchable<U> 
{
    // Add a watchable object to the watcher
    // We can call this from multiple threads
    pub fn add(&self, data: T) -> WatcherEntryInsert<U, T> {
        let idx = self.watchables.get_next_idx_increment();
        WatcherEntryInsert::new(data, idx)
    }
    // We receive a new WatcherEntry that we must watch
    pub fn received(&mut self, val: WatcherEntryInsert<U, T>) {
        self.watchables.insert(val.idx, val.data);
    }
    // Update the watcher, checking each value if it has become valid. If values did become valid, we must store their UID in self, since we will share self 
    // Update this at the end of each frame, after we do everything task related on the main thread
    pub fn update(&mut self, context: &U) {
        // Detect the values that became valid
        let validated = self.watchables.my_drain(|idx, val| val.is_valid(context)).collect::<Vec<_>>();
        // And we will set the values that became valid
        self.valids = validated.iter().map(|(idx, _)| *idx).collect::<HashSet<usize>>();
    }
    // Check if a value became valid (This can be called on any thread, as long as we have a reference to self)
    pub fn has_become_valid(&self, watchable: &WatcherEntry<U, T>) -> bool {
        self.valids.contains(&watchable.idx)
    }
}