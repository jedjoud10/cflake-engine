use ahash::AHashMap;
pub use ecs_derive::Component;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::{any::TypeId, borrow::Cow};
use crate::mask::{RawBitMask, Mask, MaskHashMap};

/// This is a certified hood classic
pub trait Component
where
    Self: 'static,
{
}

/// Default name component
#[derive(Component)]
pub struct Named(pub Cow<'static, str>);

/// Default tag component
#[derive(Component)]
pub struct Tagged(pub Cow<'static, str>);

// Registered components
lazy_static! {
    static ref NEXT: Mutex<Mask> = Mutex::new(Mask::one());
    static ref REGISTERED: RwLock<AHashMap<TypeId, Mask>> = RwLock::new(AHashMap::new());
    static ref NAMES: RwLock<MaskHashMap<String>> = RwLock::new(MaskHashMap::default());
}

/// Return the registered mask of the component (or register it if needed).
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
        let name = utils::pretty_type_name::pretty_type_name::<T>();
        locked.insert(TypeId::of::<T>(), copy);
        NAMES.write().insert(copy, name.clone());
        const ERR: &str = "Ran out of component bits to use!
        Use the 'extended-bitmasks' feature to add more bits in the bitmask if needed";
        *bit = RawBitMask::from(copy).checked_shl(1).expect(ERR).into();

        log::debug!(
            "Registered component '{name}' with bitmask 1<<{:?}",
            (copy.offset().unwrap() + 1)
        );

        copy
    }
}

/// Get the name of a component mask.
pub fn name(mask: Mask) -> Option<String> {
    if mask.count_ones() != 1 {
        return None;
    }

    let names = NAMES.read();
    names.get(&mask).cloned()
}

/// Get the number of registered components.
pub fn count() -> usize {
    REGISTERED.read().len()
}
