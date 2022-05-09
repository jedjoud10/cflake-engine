use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::Arc,
};

use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

// Trait implement for objects that can be stored within the pipeline
pub trait Cached: 'static {}

// Type aliases cause I'm cool
type ToRemove = Mutex<Vec<DefaultKey>>;
type Counters = RwLock<SecondaryMap<DefaultKey, usize>>;
type Trackers = Arc<(ToRemove, Counters)>;

// A collection of a single type of cached objects
struct SingleRow<T> {
    slotmap: SlotMap<DefaultKey, T>,
    trackers: Trackers,
}

// This shall be implement for boxed single rows
trait GenericSingleRow {
    // Remove the elements that are referenced by the given keys
    fn cleanse(&mut self);

    // Conversion to any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Cached> GenericSingleRow for SingleRow<T> {
    fn cleanse(&mut self) {
        // Simply remove the keys from the row
        let remove = std::mem::take(&mut *self.trackers.0.lock());
        let mut counters = self.trackers.1.write();
        for key in remove {
            // Silently ignore keys that have already been removed
            self.slotmap.remove(key);
            counters.remove(key);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// This is the global pipeline storage that we will use to store the multiple cached objects
#[derive(Default)]
pub(super) struct PipelineStorage {
    hashmap: AHashMap<TypeId, Box<dyn GenericSingleRow>>,
}

impl PipelineStorage {
    // Cleanse the pipeline storage, making sure to drop any objects that no longer have valid references
    pub fn cleanse(&mut self) {
        for (_, boxed) in &mut self.hashmap {
            boxed.cleanse();
        }
    }

    // Insert an object into the storage and cache it
    pub fn insert<T: Cached>(&mut self, object: T) -> Handle<T> {
        // Make sure the row is valid
        let boxed = self.hashmap.entry(TypeId::of::<T>()).or_insert_with(|| {
            Box::new(SingleRow::<T> {
                slotmap: Default::default(),
                trackers: Default::default(),
            })
        });

        // Dynamic casting from the generic row to the actual row struct
        let row = boxed.as_any_mut().downcast_mut::<SingleRow<T>>().unwrap();

        // Insert le item
        let key = row.slotmap.insert(object);
        row.trackers.1.write().insert(key, 1);

        // Construct the safe handle
        Handle {
            key,
            trackers: Some(row.trackers.clone()),
            _phantom: Default::default(),
        }
    }

    // Get an object immutably
    pub fn get<T: Cached>(&self, handle: &Handle<T>) -> Option<&T> {
        self.hashmap
            .get(&TypeId::of::<T>()).and_then(|boxed| boxed.as_any().downcast_ref::<SingleRow<T>>().unwrap().slotmap.get(handle.key))
    }

    // Get an object mutably
    pub fn get_mut<T: Cached>(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.hashmap
            .get_mut(&TypeId::of::<T>()).and_then(|boxed| boxed.as_any_mut().downcast_mut::<SingleRow<T>>().unwrap().slotmap.get_mut(handle.key))
    }
}

// A strong object handle that we can use to reference an object within the pipeline
pub struct Handle<T: Cached> {
    key: DefaultKey,
    trackers: Option<Trackers>,
    _phantom: PhantomData<T>,
}

impl<T: Cached> Handle<T> {
    // Try to get the number of strong handles that are currently referencing our cached object
    pub fn count_strong(&self) -> usize {
        self.trackers.as_ref().map(|trackers| trackers.1.read()[self.key]).unwrap_or_default()
    }
}

impl<T: Cached> Clone for Handle<T> {
    fn clone(&self) -> Self {
        // Increment the strong handle counter if possible
        if let Some(trackers) = self.trackers.as_ref() {
            let counter = &mut trackers.1.write()[self.key];
            *counter = counter.saturating_add(1);
        }

        Self {
            key: self.key,
            trackers: self.trackers.clone(),
            _phantom: Default::default(),
        }
    }
}

impl<T: Cached> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(trackers) = self.trackers.as_ref() {
            // Decrement the strong handle counter if possible
            let counter = &mut trackers.1.write()[self.key];
            *counter = counter.saturating_sub(1);

            // If we have no strong handles, we must deallocate the object
            if *counter == 0 {
                trackers.0.lock().push(self.key);
            }
        }
    }
}
