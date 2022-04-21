use super::{Handle, PipelineElemKey};
use crate::object::Object;
use ahash::AHashMap;
use parking_lot::Mutex;
use slotmap::SlotMap;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::Arc,
};

// Le type alias
type ElemMap<U> = SlotMap<PipelineElemKey, U>;
type ToRemoveMap = Arc<Mutex<Vec<PipelineElemKey>>>;
type CollectionTuple<U> = (ElemMap<U>, ToRemoveMap);

// Trait that will be implemented for the collection tuple
trait Collection {
    // "As any" and "as any mut" for conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Remove all the dangling elements
    fn cleanse(&mut self);
}

impl<U: Object> Collection for CollectionTuple<U> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn cleanse(&mut self) {
        let mut to_remove_locked = self.1.lock();
        let to_remove = std::mem::take(&mut *to_remove_locked);
        for key in to_remove {
            // Silently ignore elements that have already been removed
            if let Some(removed) = self.0.remove(key) {
                removed.disposed()
            }
        }
    }
}

// Contains multiple collections
#[derive(Default)]
pub(super) struct PipelineStorage {
    hashmap: AHashMap<TypeId, Box<dyn Collection>>,
}

impl PipelineStorage {
    // Cleanse all of the collections
    pub fn cleanse(&mut self) {
        self.hashmap.iter_mut().for_each(|(_, boxed)| boxed.cleanse());
    }

    // Add a new object, and create it's unique collection
    pub fn insert<U: Object>(&mut self, object: U) -> Handle<U> {
        // Cast the boxed collection to it's mutable reference
        let boxed = self.hashmap.entry(TypeId::of::<U>()).or_insert_with(|| Box::new(CollectionTuple::<U>::default()));
        let any = boxed.as_any_mut();
        let (slotmap, to_remove) = any.downcast_mut::<CollectionTuple<U>>().unwrap();
        let key = Arc::new(slotmap.insert(object));

        // Create the handle from the key
        Handle {
            key,
            to_remove: Some(to_remove.clone()),
            _phantom: PhantomData::default(),
        }
    }

    // Immutable getter
    pub fn get<U: Object>(&self, handle: &Handle<U>) -> Option<&U> {
        let id = TypeId::of::<U>();
        self.hashmap.get(&id).map(|boxed| {
            let any = boxed.as_any();
            let (slotmap, _) = any.downcast_ref::<CollectionTuple<U>>().unwrap();
            slotmap.get(*handle.key.as_ref()).unwrap()
        })
    }

    // Mutable getter
    pub fn get_mut<U: Object>(&mut self, handle: &Handle<U>) -> Option<&mut U> {
        let id = TypeId::of::<U>();
        self.hashmap.get_mut(&id).map(|boxed| {
            let any = boxed.as_any_mut();
            let (slotmap, _) = any.downcast_mut::<CollectionTuple<U>>().unwrap();
            slotmap.get_mut(*handle.key.as_ref()).unwrap()
        })
    }
}
