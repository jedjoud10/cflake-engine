use crate::utils::ComponentError;
use bitfield::Bitfield;
use lazy_static::lazy_static;
use std::{
    any::Any,
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering, AtomicU8},
        RwLock,
    },
};

use super::{Component, GlobalComponent};
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);
    static ref NEXT_REGISTERED_GLOBAL_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);
    static ref REGISTERED_COMPONENTS: RwLock<HashMap<String, Bitfield<u32>>> = RwLock::new(HashMap::new());
    static ref REGISTERED_GLOBAL_COMPONENTS: RwLock<HashMap<String, Bitfield<u32>>> = RwLock::new(HashMap::new());
}

// Register a specific component
pub fn register_component<T: Component + Sized>() -> Bitfield<u32> {
    // Register the component
    let mut rc = REGISTERED_COMPONENTS.write().unwrap();
    // Make a copy of the id before the bit shift
    let id = NEXT_REGISTERED_COMPONENT_ID.load(Ordering::Relaxed);

    let component_id = Bitfield::<u32>::from_num(id);
    rc.insert(T::get_component_name(), component_id);
    // Bit shift to the left
    NEXT_REGISTERED_COMPONENT_ID.store(id << 1, Ordering::Relaxed);
    println!("{} {}", T::get_component_name(), component_id);
    // Return the component ID before the bit shift
    component_id
}
// Get the bitfield ID of a specific component
pub fn get_component_bitfield<T: Component>() -> Bitfield<u32> {
    if is_component_registered::<T>() {
        // Simple read
        let rc = REGISTERED_COMPONENTS.read().unwrap();
        rc[&T::get_component_name()]
    } else {
        // Register if it wasn't registered yet
        register_component::<T>()
    }
}
// Checks if a specific component is registered
pub fn is_component_registered<T: Component>() -> bool {
    let rc = REGISTERED_COMPONENTS.read().unwrap();
    rc.contains_key(&T::get_component_name())
}
// Get multiple component names from a cBitfield
pub fn get_component_names_cbitfield(cbitfield: Bitfield<u32>) -> Vec<String> {
    let read = REGISTERED_COMPONENTS.read().unwrap();
    let mut component_names = Vec::new();
    for (component_name, id) in (*read).iter() {
        if cbitfield.contains(id) {
            component_names.push(component_name.clone());
        }
    }
    component_names
}

// Cast a boxed component to a reference of that component
pub(crate) fn cast_component<'a, T>(linked_component: &'a dyn Component) -> Result<&T, ComponentError>
where
    T: Component + Send + Sync + 'static,
{
    let component_any: &dyn Any = linked_component.as_any();
    let reference = component_any
        .downcast_ref::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed component to a mutable reference of that component
pub(crate) fn cast_component_mut<'a, T>(linked_component: &'a mut dyn Component) -> Result<&mut T, ComponentError>
where
    T: Component + Send + Sync + 'static,
{
    let component_any: &mut dyn Any = linked_component.as_any_mut();
    let reference_mut = component_any
        .downcast_mut::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference_mut)
}


// Register a specific global component
pub fn register_global_component<T: GlobalComponent + Sized>() -> Bitfield<u32> {
    // Register the component
    let mut rc = REGISTERED_GLOBAL_COMPONENTS.write().unwrap();
    // Make a copy of the id before the bit shift
    let id = NEXT_REGISTERED_GLOBAL_COMPONENT_ID.load(Ordering::Relaxed);

    let component_id = Bitfield::<u32>::from_num(id);
    rc.insert(T::get_component_name(), component_id);
    // Bit shift to the left
    NEXT_REGISTERED_GLOBAL_COMPONENT_ID.store(id << 1, Ordering::Relaxed);
    println!("{} {}", T::get_component_name(), component_id);
    // Return the component ID before the bit shift
    component_id
}
// Get the bitfield ID of a specific global component
pub fn get_global_component_bitfield<T: GlobalComponent>() -> Bitfield<u32> {
    if is_global_component_registered::<T>() {
        // Simple read
        let rc = REGISTERED_GLOBAL_COMPONENTS.read().unwrap();
        rc[&T::get_component_name()]
    } else {
        // Register if it wasn't registered yet
        register_global_component::<T>()
    }
}
// Checks if a specific global component is registered
pub fn is_global_component_registered<T: GlobalComponent>() -> bool {
    let rc = REGISTERED_GLOBAL_COMPONENTS.read().unwrap();
    rc.contains_key(&T::get_component_name())
}

// Cast a boxed global component to a reference of that global component
pub(crate) fn cast_global_component<'a, T>(linked_component: &'a dyn GlobalComponent) -> Result<&T, ComponentError>
where
    T: GlobalComponent + 'static,
{
    let component_any: &dyn Any = linked_component.as_any();
    let reference = component_any
        .downcast_ref::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed global component to a mutable reference of that global component
pub(crate) fn cast_global_component_mut<'a, T>(linked_component: &'a mut dyn GlobalComponent) -> Result<&mut T, ComponentError>
where
    T: GlobalComponent + 'static,
{
    let component_any: &mut dyn Any = linked_component.as_any_mut();
    let reference_mut = component_any
        .downcast_mut::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference_mut)
}