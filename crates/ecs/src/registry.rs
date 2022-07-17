use crate::Mask;
use ahash::AHashMap;
pub use ecs_derive::Component;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::any::{type_name, TypeId};

// This is a certified hood classic
pub trait Component
where
    Self: 'static + Sized,
{
}

// Registered components
lazy_static! {
    static ref NEXT: Mutex<Mask> = Mutex::new(Mask::one());
    static ref REGISTERED: RwLock<AHashMap<TypeId, Mask>> = RwLock::new(AHashMap::new());
}

// Return the registered mask of the component (or register it if needed)
pub fn mask<T: Component>() -> Mask {
    // Check if we need to register
    let id = TypeId::of::<T>();
    if REGISTERED.read().contains_key(&id) {
        // Read normally
        let locked = REGISTERED.read();
        *locked.get(&id).unwrap()
    } else {
        // Register the component
        register::<T>()
    }
}
// Registers the component manually
pub fn register<T: Component>() -> Mask {
    let mut locked = REGISTERED.write();
    let mut bit = NEXT.lock();

    // Le bitshifting
    let copy = *bit;
    locked.insert(TypeId::of::<T>(), copy);
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
