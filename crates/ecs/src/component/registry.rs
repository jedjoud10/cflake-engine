use std::{any::{TypeId, type_name}, collections::HashMap};

use lazy_static::lazy_static;
use parking_lot::{RwLock, Mutex};
use super::{Component, ComponentError};
// Registered components
lazy_static! {
    static ref NEXT: Mutex<u64> = Mutex::new(1);
    static ref REGISTERED: RwLock<HashMap<TypeId, u64>> = RwLock::new(HashMap::new());
}
// Return the registered bits of the component
pub fn bits<T: Component>() -> Result<u64, ComponentError> {
    let locked = REGISTERED.read();
    let id = TypeId::of::<T>();
    locked.get(&id).ok_or(ComponentError::NotRegistered(type_name::<T>())).cloned()
}
// Registers the component if it wasn't already registered.
// This is a no op if the component is already registered
pub fn register<T: Component>() -> u64 {
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