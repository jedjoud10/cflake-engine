use crate::{Archetype, Component, LayoutAccess, mask, Mask};

// Mutable query layouts that might contain mutable references
// This must take a mutable reference to the current archetype 
pub trait MutQueryLayout<'a>: 'a + Sized {
    type Slices: 'a;
    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Slices>;
    fn is_valid() -> bool;
    fn read(slices: &'a mut Self::Slices, i: usize) -> Self;
}

// Immutable query layouts that will never contain any mutable referneces
// This simply takes an immutable reference to the archetype
pub trait RefQueryLayout<'a>: 'a + Sized {
    type Slices: 'a;
    fn prepare(archetype: &'a Archetype) -> Option<Self::Slices>;
    fn is_valid() -> bool;
    fn read(slices: &'a Self::Slices, i: usize) -> Self;
}

// Immutable component items that will be stored within the archetype
pub trait RefQueryItem<'a>: 'a + Sized {
    type Item: 'static + Component;
    fn access() -> LayoutAccess;
    fn get(archetype: &'a Archetype) -> Option<&'a [Self::Item]>;
    fn read(slice: &'a [Self::Item], i: usize) -> Self;
}

// Implementations of ref query item for &T
impl<'a, T: Component> RefQueryItem<'a> for &'a T {
    type Item = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    fn get(archetype: &'a Archetype) -> Option<&'a [Self::Item]> {
        archetype.table::<T>().map(|vec| vec.as_slice())
    }

    fn read(slice: &'a [Self::Item], i: usize) -> Self {
        &slice[i]
    }
} 

// Mutable component items that will be stored within the archetypes
pub trait MutQueryItem<'a>: 'a + Sized {
    type Item: 'static + Component;
    fn access() -> LayoutAccess;
    fn get(archetype: &'a mut Archetype) -> Option<&'a mut [Self::Item]>;
    fn read(slice: &'a mut [Self::Item], i: usize) -> Self;
}

// Implementations of mut query item for &T
impl<'a, T: Component> MutQueryItem<'a> for &'a T {
    type Item = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    fn get(archetype: &'a mut Archetype) -> Option<&'a mut [Self::Item]> {
        archetype.table_mut::<T>().map(|vec| vec.as_mut_slice())
    }

    fn read(slice: &'a mut [Self::Item], i: usize) -> Self {
        &slice[i]
    }
} 

// Implementations of mut query item for &mut T
impl<'a, T: Component> MutQueryItem<'a> for &'a mut T {
    type Item = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    fn get(archetype: &'a mut Archetype) -> Option<&'a mut [Self::Item]> {
        archetype.table_mut::<T>().map(|vec| vec.as_mut_slice())
    }

    fn read(slice: &'a mut [Self::Item], i: usize) -> Self {
        &mut slice[i]
    }
} 