use std::sync::{atomic::{AtomicU16, Ordering::Relaxed}, RwLock};

use ordered_vec::ordered_vec::OrderedVec;
static GLOBAL_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

// A command ID
// Each time we tell the Render Pipeline or the World to execute something (Create an Entity, Create a Texture, etc...) we will increment the global command ID counter
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandID {
    pub(crate) id: u16,
}

impl CommandID {
    pub fn new() -> Self {
        Self { 
            id: GLOBAL_ID_COUNTER.fetch_add(1, Relaxed)
        }
    }
}

// A watchable trait. Implemented on structs that can be detected whenever they execute
pub trait Watchable {
    fn get_id(&self) -> CommandID;
    fn is_valid(&self) -> bool;
}

// A thread local watcher that can detect when certain "things" happen
pub struct Watcher<T> 
    where T: Watchable
{
    watchables: OrderedVec<T>, // The watchable values that are waiting to become valid
    valids: RwLock<Vec<CommandID>>, // The watchable values that have become valid in the next frame. We clear this buffer at the end of each frame
}

impl<T> Watcher<T>
    where T: Watchable
{
    // Add a watchable object to the watcher
    pub fn add(&mut self, watchable: T) {
        self.watchables.push_shove(watchable);
    }
    // Update the watcher, checking each value if it has become valid. If values did become valid, we must store their CommandID in a RwLock so we can share it around 
    // Update this at the end of each frame
    pub fn update(&mut self) {
        // Detect the values that became valid
        let validated = self.watchables.my_drain(|index, val| val.is_valid()).collect::<Vec<(usize, T)>>();
        let mut valids = self.valids.write().unwrap();
        valids.clear();
        *valids = validated.iter().map(|(index, val)| val.get_id()).collect::<Vec<CommandID>>();
    }
    // Check if a value became valid (This can be called on any thread, as long as we have a reference to self)
    pub fn has_become_valid(&self, watchable: T) -> bool {
        let valids = self.valids.read().unwrap();
        valids.contains(&watchable.get_id())
    }
}