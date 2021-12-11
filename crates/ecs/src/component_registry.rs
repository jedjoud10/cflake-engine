use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
};

use crate::{ComponentID, ECSError};
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicUsize = AtomicUsize::new(0);
    static ref REGISTERED_COMPONENTS: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
}

// Register a specific component
pub fn register_component<T: ComponentID>() -> usize {
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
pub fn get_component_id<T: ComponentID>() -> Result<usize, ECSError> {
    let name: String = T::get_component_name();
    let rc = REGISTERED_COMPONENTS.read().unwrap();
    // It found the component, so just return it's id
    if rc.contains_key(&name) {
        let value = rc[&name];
        Ok(value)
    } else {
        return Err(ECSError::new(format!("Component {} not registered!", name)));
    }
}
// Checks if a specific component is registered
pub fn is_component_registered<T: ComponentID>() -> bool {
    let rc = REGISTERED_COMPONENTS.read().unwrap();
    rc.contains_key(&T::get_component_name())
}
