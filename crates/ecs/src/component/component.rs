use getset::CopyGetters;
use lazy_static::lazy_static;
use parking_lot::{MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard};
use std::{
    any::{type_name, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
    sync::Arc,
};

use crate::{
    archetype::{ArchetypeId, ArchetypeSet, ComponentsHashMap, MaybeNoneStorage, NoHash},
    manager::EcsManager,
};

use super::ComponentError;

// Registered components
lazy_static! {
    static ref NEXT: Mutex<u64> = Mutex::new(1);
    static ref REGISTERED: RwLock<HashMap<TypeId, u64>> = RwLock::new(HashMap::new());
}

// Implemented for components
pub trait Component
where
    Self: 'static + Sync + Send,
{
    // Return the registered bits of the component
    fn bits() -> Result<u64, ComponentError> {
        let locked = REGISTERED.read();
        let id = TypeId::of::<Self>();
        locked
            .get(&id)
            .ok_or(ComponentError::NotRegistered(type_name::<Self>()))
            .cloned()
    }
    // Registers the component if it wasn't already registered.
    // This is a no op if the component is already registered
    fn register() -> u64 {
        let mut locked = REGISTERED.write();
        let id = TypeId::of::<Self>();
        // If the component was already registered, no need to do anything
        if let Some(&bits) = locked.get(&id) {
            return bits;
        }

        // Left bitshft
        let mut bit = NEXT.lock();
        // Keep a copy of bit
        let copy = *bit;
        locked.insert(id, copy);
        *bit = *bit << 1;
        copy
    }
}

impl<T> Component for T where T: 'static + Sized + Send + Sync {}
