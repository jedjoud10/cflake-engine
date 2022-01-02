use std::{sync::{atomic::{AtomicPtr, Ordering::{Relaxed, self}, AtomicUsize}, Arc, RwLock}, ptr::null_mut, marker::PhantomData, collections::HashMap};
use lazy_static::lazy_static;
use ordered_vec::ordered_vec::OrderedVec;

// We will have a main global buffer, holding the state of each ExternalID
// If it receives a message saying that a specific ExternalID became valid, we can remove it from the global buffer

// Counter
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Global buffer holding the state of each ExternalID
pub struct GlobalBuffer<T, U>
    where T: ExternalID<U>,
{
    // The state of each ExternalID
    states: HashMap<usize, U>,
    phantom: PhantomData<T>,
}

impl<T, U> Default for GlobalBuffer<T, U>
    where T: ExternalID<U>,
{
    fn default() -> Self {
        Self { 
            states: HashMap::default(),
            phantom: PhantomData::default()
        }
    }
}

impl<T, U> GlobalBuffer<T, U>
    where T: ExternalID<U>,
{
    // Clear the buffer at the end of every frame
    pub fn clear(&mut self) {
        self.states.clear();
    }
    // We have received the internal value for an ExternalID
    pub(crate) fn receive(&mut self, external_id: T, internal_val: U) {
        self.states.insert(external_id.id(), internal_val);
    }
    // Remove an internal value
    pub(crate) fn remove(&mut self, external_id: T) {
        self.states.remove(&external_id.id());
    }
    // Check if an ExternalID can be valid
    pub(crate) fn poll(&self, external_id: &T) -> Option<&U> {
        if self.states.contains_key(&external_id.id()) {
            // We have the key, we can return the value
            Some(self.states.get(&external_id.id()).unwrap())
        } else {
            // Non
            None
        }
    }
}


// Some sort of external ID that we can generate on other threads, and that will be passed to the main thread whenever we actually want to generate the ID
pub trait ExternalID<T>
    where Self: Sized
{
    // Create a new ExternalID
    fn new() -> Self;
    // Get the internal ID
    fn id(&self) -> usize;
    // Increment
    fn increment() -> usize {
        ID_COUNTER.fetch_add(1, Relaxed)
    }
    // Update the internal value of the ExternalID. WE MUST DO THIS ON THE MAIN THREAD
    // We can only do this once, after we update the internal value of the ExternalID, we will not be able to update it no more
    fn set(self, internal_val: T, buffer: &mut GlobalBuffer<Self, T>) {
        // Set the value if it doesn't exist
        if let None = buffer.poll(&self) {
            buffer.receive(self, internal_val)
        }
    }
    // Invalidate
    fn invalidate(self, buffer: &mut GlobalBuffer<Self, T>) {
        buffer.remove(self);
    }
    // Check if the internal value has been returned, and try to get it.
    fn try_get<'a>(&self, buffer: &'a GlobalBuffer<Self, T>) -> Option<&'a T> {
        buffer.poll(&self) 
    }
}