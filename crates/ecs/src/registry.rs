use crate::{Mask, RawBitMask};
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
    static ref REGISTERED: RwLock<AHashMap<TypeId, Mask>> =
        RwLock::new(AHashMap::new());
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
        let mut locked = REGISTERED.write();
        let mut bit = NEXT.lock();

        // Le bitshifting
        let copy = *bit;
        locked.insert(TypeId::of::<T>(), copy);
        const ERR: &str = "Ran out of component bits to use!
        Use the 'extended-bitmasks' feature to add more bits in the bitmask if needed";
        *bit =
            RawBitMask::from(copy).checked_shl(1).expect(ERR).into();
        copy
    }
}

// Get the name of a component
pub fn name<T: Component>() -> &'static str {
    type_name::<T>()
}

// Get the number of registered components
pub fn count() -> usize {
    REGISTERED.read().len()
}
