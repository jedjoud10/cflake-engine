use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
};

use crate::{Component};
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicUsize = AtomicUsize::new(1);
    static ref REGISTERED_COMPONENTS: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
}

// Register a specific component
pub fn register_component<T: Component + Sized>() -> usize {
    // Register the component
    let mut rc = REGISTERED_COMPONENTS.write().unwrap();
    let id = NEXT_REGISTERED_COMPONENT_ID.load(Ordering::Relaxed);
    rc.insert(T::get_component_name(), id);
    // Make a copy of the id before the bit shift
    let component_id = id;
    // Bit shift to the left
    NEXT_REGISTERED_COMPONENT_ID.store(component_id << 1, Ordering::Relaxed);
    // Return the component id before the bit shift
    component_id
}
// Get the bitfield ID of a specific component
pub fn get_component_id<T: Component>() -> usize {
    if is_component_registered::<T>() {
        let rc = REGISTERED_COMPONENTS.read().unwrap();
        let value = rc[&T::get_component_name()];
        value
    } else {
        register_component::<T>()
    }
}
// Checks if a specific component is registered
pub fn is_component_registered<T: Component>() -> bool {
    let rc = REGISTERED_COMPONENTS.read().unwrap();
    rc.contains_key(&T::get_component_name())
}
// Get multiple component names from a cBitfield
pub fn get_component_names_cbitfield(cbitfield: usize) -> Vec<String> {
    let read = REGISTERED_COMPONENTS.read().unwrap();
    let mut component_names = Vec::new();
    for (component_name, id) in (*read).iter() {
        // If it is valid
        if (id & !cbitfield) == 0 {
            component_names.push(component_name.clone());
        }
    }
    component_names
}
