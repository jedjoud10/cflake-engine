use crate::utils::ComponentError;
use ahash::AHashMap;
use bitfield::Bitfield;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::{
    any::{Any, TypeId},
    sync::atomic::{AtomicU32, Ordering, AtomicBool},
};

use super::Component;
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);
    static ref CAN_REGISTER: AtomicBool = AtomicBool::new(true);
    static ref REGISTERED_COMPONENTS: RwLock<AHashMap<TypeId, Bitfield<u32>>> = RwLock::new(AHashMap::new());
}

// Disable component registration
pub(super) fn disable() {
    CAN_REGISTER.store(false, Ordering::Relaxed)
}

// Register a specific component
pub fn register<T: Component + Sized>() -> Bitfield<u32> {
    // Check first
    assert!(CAN_REGISTER.load(Ordering::Relaxed), "Cannot register components during frame!");

    // Register the component
    let mut lock = REGISTERED_COMPONENTS.write();
    // Make a copy of the id before the bit shift
    let id = NEXT_REGISTERED_COMPONENT_ID.load(Ordering::Relaxed);

    let cbitfield = Bitfield::<u32>::from_num(id);
    lock.insert(TypeId::of::<T>(), cbitfield);
    // Bit shift to the left
    NEXT_REGISTERED_COMPONENT_ID.store(id << 1, Ordering::Relaxed);
    // Return the component ID before the bit shift
    eprintln!("Registered component '{}' with bitfield '{}'", std::any::type_name::<T>(), cbitfield);
    cbitfield
}
// Get the bitfield ID of a specific component
pub fn get<T: Component>() -> Bitfield<u32> {
    let is_registered = REGISTERED_COMPONENTS.read().contains_key(&TypeId::of::<T>());
    if is_registered {
        // Simple read
        let lock = REGISTERED_COMPONENTS.read();
        lock[&TypeId::of::<T>()]
    } else {
        // Register the component ourselves
        register::<T>()
    }
}
// Cast a boxed component to a reference of that component
pub(crate) fn cast<T>(component: &dyn Component) -> Result<&T, ComponentError>
where
    T: Component,
{
    let component_any: &dyn Any = component.as_any();
    let reference = component_any
        .downcast_ref::<T>()
        .ok_or_else(|| ComponentError::new("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed component to a mutable reference of that component
pub(crate) fn cast_mut<T>(component: &mut dyn Component) -> Result<&mut T, ComponentError>
where
    T: Component,
{
    let component_any: &mut dyn Any = component.as_any_mut();
    let reference_mut = component_any
        .downcast_mut::<T>()
        .ok_or_else(|| ComponentError::new("Could not cast component".to_string()))?;
    Ok(reference_mut)
}
