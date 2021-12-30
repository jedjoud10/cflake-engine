use bitfield::BitfieldU32;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        RwLock,
    },
};

use crate::{bitfield::ComponentBitfield, Component};
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);
    static ref REGISTERED_COMPONENTS: RwLock<HashMap<String, ComponentBitfield>> = RwLock::new(HashMap::new());
}

// Register a specific component
pub fn register_component<T: Component + Sized>() -> ComponentBitfield {
    // Register the component
    let mut rc = REGISTERED_COMPONENTS.write().unwrap();
    // Make a copy of the id before the bit shift
    let id = NEXT_REGISTERED_COMPONENT_ID.load(Ordering::Relaxed);

    let component_id = ComponentBitfield {
        bitfield: BitfieldU32::from_num(id),
    };
    rc.insert(T::get_component_name(), component_id.clone());
    // Bit shift to the left
    NEXT_REGISTERED_COMPONENT_ID.store(id << 1, Ordering::Relaxed);
    // Return the component ID before the bit shift
    component_id
}
// Get the bitfield ID of a specific component
pub fn get_component_bitfield<T: Component>() -> ComponentBitfield {
    if is_component_registered::<T>() {
        let rc = REGISTERED_COMPONENTS.read().unwrap();
        let value = rc[&T::get_component_name()].clone();
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
pub fn get_component_names_cbitfield(cbitfield: ComponentBitfield) -> Vec<String> {
    let read = REGISTERED_COMPONENTS.read().unwrap();
    let mut component_names = Vec::new();
    for (component_name, id) in (*read).iter() {
        if cbitfield.bitfield.contains(&id.bitfield) {
            component_names.push(component_name.clone());
        }
    }
    component_names
}
