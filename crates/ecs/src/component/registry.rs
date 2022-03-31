use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::{archetype::ComponentStorage, Mask};

use super::{Component, ComponentError};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
// Registered components
lazy_static! {
    static ref NEXT: Mutex<Mask> = Mutex::new(Mask(1));
    static ref REGISTERED: RwLock<HashMap<TypeId, Mask>> = RwLock::new(HashMap::new());
}
// Return the registered mask of the component
#[inline(always)]
pub fn mask<T: Component>() -> Result<Mask, ComponentError> {
    let locked = REGISTERED.read();
    let id = TypeId::of::<T>();
    locked.get(&id).ok_or(ComponentError::NotRegistered(name::<T>())).cloned()
}
// Registers the component if it wasn't already registered
#[inline(always)]
pub fn register<T: Component>() -> Mask {
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
#[inline(always)]
pub fn name<T: Component>() -> &'static str {
    type_name::<T>()
}
