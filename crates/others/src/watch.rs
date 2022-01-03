use std::{sync::{atomic::{AtomicU16, Ordering::Relaxed}, RwLock}, marker::PhantomData};
use ordered_vec::ordered_vec::OrderedVec;

use crate::ExternalID;

// A watchable trait. Implemented on structs that can be detected whenever they execute
pub trait Watchable<U> {
    fn get_uid(&self) -> usize;
    fn is_valid(&self, context: &U) -> bool;
}

// A thread local watcher that can detect when certain "things" happen
pub struct Watcher<T, U> 
    where T: Watchable<U>
{
    phantom: PhantomData<U>,
    watchables: OrderedVec<T>, // The watchable values that are waiting to become valid
    valids: RwLock<Vec<usize>>, // The watchable values that have become valid in the next frame. We clear this buffer at the end of each frame
}

impl<T, U> Default for  Watcher<T, U>
    where T: Watchable<U>
{
    fn default() -> Self {
        Self { 
            phantom: PhantomData::default(),
            watchables: OrderedVec::default(),
            valids: RwLock::new(Vec::new())
        }
    }
}



impl<T, U> Watcher<T, U>
    where T: Watchable<U>
{
    // Add a watchable object to the watcher
    pub fn add(&mut self, watchable: T) {
        self.watchables.push_shove(watchable);
    }
    // Update the watcher, checking each value if it has become valid. If values did become valid, we must store their CommandID in a RwLock so we can share it around 
    // Update this at the end of each frame
    pub fn update(&mut self, context: &U) {
        // Detect the values that became valid
        let validated = self.watchables.my_drain(|index, val| val.is_valid(context)).collect::<Vec<(usize, T)>>();
        let mut valids = self.valids.write().unwrap();
        valids.clear();
        *valids = validated.iter().map(|(index, val)| val.get_uid()).collect::<Vec<usize>>();
    }
    // Check if a value became valid (This can be called on any thread, as long as we have a reference to self)
    pub fn has_become_valid(&self, watchable: &T) -> bool {
        let valids = self.valids.read().unwrap();
        valids.contains(&watchable.get_uid())
    }
}