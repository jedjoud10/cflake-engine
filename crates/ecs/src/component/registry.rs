use crate::utils::ComponentError;
use ahash::AHashMap;
use bitfield::Bitfield;
use lazy_static::lazy_static;
use std::{
    any::{Any, TypeId},
    sync::{
        atomic::{AtomicU32, Ordering},
        RwLock,
    },
};

use super::Component;
// Use to keep track of the component IDs
lazy_static! {
    static ref NEXT_REGISTERED_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);
    static ref REGISTERED_COMPONENTS: RwLock<AHashMap<TypeId, Bitfield<u32>>> =
        RwLock::new(AHashMap::new());
}

// Register a specific component
pub fn register_component<T: Component + Sized + 'static>() -> Bitfield<u32> {
    // Register the component
    let mut rc = REGISTERED_COMPONENTS.write().unwrap();
    // Make a copy of the id before the bit shift
    let id = NEXT_REGISTERED_COMPONENT_ID.load(Ordering::Relaxed);

    let component_id = Bitfield::<u32>::from_num(id);
    rc.insert(TypeId::of::<T>(), component_id);
    // Bit shift to the left
    NEXT_REGISTERED_COMPONENT_ID.store(id << 1, Ordering::Relaxed);
    log::info!("{:?} {}", TypeId::of::<T>(), component_id);
    // Return the component ID before the bit shift
    component_id
}
// Get the bitfield ID of a specific component
pub fn get_component_bitfield<T: Component + 'static>() -> Bitfield<u32> {
    if is_component_registered::<T>() {
        // Simple read
        let rc = REGISTERED_COMPONENTS.read().unwrap();
        rc[&TypeId::of::<T>()]
    } else {
        // Register if it wasn't registered yet
        register_component::<T>()
    }
}
// Checks if a specific component is registered
pub fn is_component_registered<T: Component + 'static>() -> bool {
    let rc = REGISTERED_COMPONENTS.read().unwrap();
    rc.contains_key(&TypeId::of::<T>())
}
// Cast a boxed component to a reference of that component
pub(crate) fn cast_component<T>(component: &dyn Component) -> Result<&T, ComponentError>
where
    T: Component + 'static,
{
    let component_any: &dyn Any = component.as_any();
    let reference = component_any
        .downcast_ref::<T>()
        .ok_or_else(|| ComponentError::new("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed component to a mutable reference of that component
pub(crate) fn cast_component_mut<T>(component: &mut dyn Component) -> Result<&mut T, ComponentError>
where
    T: Component + 'static,
{
    let component_any: &mut dyn Any = component.as_any_mut();
    let reference_mut = component_any
        .downcast_mut::<T>()
        .ok_or_else(|| ComponentError::new("Could not cast component".to_string()))?;
    Ok(reference_mut)
}
