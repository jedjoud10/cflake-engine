use std::{any::TypeId, sync::OnceLock};

use ahash::AHashMap;
use parking_lot::Mutex;
use world::World;

pub(crate) static PER_FRAME_EVENTS_CACHE_CLEANER: OnceLock<
    Mutex<AHashMap<TypeId, Box<dyn Fn(&World) + Send>>>,
> = OnceLock::new();

// Simple utility to handle communications between systems
// Design stolen from bevy event sender / receiver
// The internal events will be cleared at the end of every frame using the clear
pub struct PerFrameEvents<T: 'static>(Vec<T>);

impl<T: 'static> PerFrameEvents<T> {
    // Create a a new per frame events manager to be used inside the world
    pub fn new() -> Self {
        let mutex = PER_FRAME_EVENTS_CACHE_CLEANER.get_or_init(|| Default::default());
        let mut locked = mutex.lock();
        locked.entry(TypeId::of::<T>()).or_insert_with(|| {
            Box::new(|world: &World| {
                if let Ok(mut events) = world.get_mut::<Self>() {
                    events.clear();
                }
            })
        });

        Self(Vec::default())
    }

    // Send some events to be stored persistently accross system boundaries
    pub fn send(&mut self, iter: impl Iterator<Item = T>) {
        self.0.extend(iter);
    }

    // Receive some events immutably (do not take them)
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.0.as_slice().iter()
    }

    // Consume all the given events
    pub fn consume(&mut self) -> impl IntoIterator<Item = T> {
        std::mem::take(&mut self.0)
    }

    // Clear the per frame events
    pub fn clear(&mut self) {
        self.0.clear();
    }
}
