use super::{Component, ComponentError};
use crate::Mask;
use ahash::AHashMap;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::{any::{type_name, TypeId}, sync::atomic::{AtomicBool, Ordering}};
// Registered components
lazy_static! {
    static ref NEXT: Mutex<Mask> = Mutex::new(Mask::one());
    static ref REGISTERED: RwLock<AHashMap<TypeId, Mask>> = RwLock::new(AHashMap::new());
}

// Return the registered mask of the component
pub fn mask<T: Component>() -> Result<Mask, ComponentError> {
    let locked = REGISTERED.read();
    let id = TypeId::of::<T>();
    locked.get(&id).ok_or(ComponentError::NotRegistered(name::<T>())).cloned()
}
// Registers the component if it wasn't already registered
pub(crate) fn register<T: Component>() -> Mask {
    let mut locked = REGISTERED.write();
    let id = TypeId::of::<T>();
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
// Get the name of a component
pub fn name<T: Component>() -> &'static str {
    type_name::<T>()
}
// Get the number of registered components
pub fn count() -> usize {
    REGISTERED.read().len()
}
