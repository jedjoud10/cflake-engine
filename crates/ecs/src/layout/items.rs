use super::LayoutAccess;
use crate::{
    archetype::Archetype,
    entity::Entity,
    mask::Mask,
    registry::{mask, Component},
};

/// Immutable query slice that will be fetched from each archetype.
pub trait QueryItemRef<'s>: Sized {
    /// Immutable slice of the query item.
    type Slice: 's + Copy;

    /// Get the layout access mask for this item.
    fn access() -> LayoutAccess;

    /// Get a slice from an immutable archetype.
    fn from_archetype(archetype: &'s Archetype) -> Self::Slice;

    /// Read from a the slice directly at a specified index.
    fn read(slice: Self::Slice, index: usize) -> Self;
}

/// Mutable query slice that will be fetched from each archetype.
pub trait QueryItemMut<'s>: Sized {
    /// Immutable slice of the query item.
    type Slice: 's;

    /// Get the layout access mask for this item.
    fn access() -> LayoutAccess;

    /// Get a slice from an mutable archetype.
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice;

    /// Read from a the slice directly at a specified index.
    fn read_mut(slice: Self::Slice, index: usize) -> Self;
}

impl<'s, T: Component> QueryItemRef<'s> for &'s T {
    type Slice = &'s [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_archetype(archetype: &'s Archetype) -> Self::Slice {
        archetype.components::<T>().unwrap().as_slice()
    }
    
    #[inline]
    fn read(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}

impl<'s, T: Component> QueryItemRef<'s> for Option<&'s T> {
    type Slice = Option<&'s [T]>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_archetype(archetype: &'s Archetype) -> Self::Slice {
        archetype
            .components::<T>()
            .map(|col| col.as_slice()) 
    }
    
    #[inline]
    fn read(slice: Self::Slice, index: usize) -> Self {
        slice.map(|slice| &slice[index])
    }
}


impl<'s> QueryItemRef<'s> for &'s Entity {
    type Slice = &'s [Entity];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    fn from_archetype(archetype: &'s Archetype) -> Self::Slice {
        archetype.entities()
    }

    #[inline]
    fn read(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}


impl<'s> QueryItemRef<'s> for &'s () {
    type Slice = &'s [()];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    fn from_archetype(archetype: &'s Archetype) -> Self::Slice {
        &[]
    }

    #[inline]
    fn read(slice: Self::Slice, index: usize) -> Self {
        &()
    }
}

impl<'s, T: Component> QueryItemMut<'s> for &'s T {
    type Slice = &'s [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype.components_mut::<T>().unwrap().as_mut_slice()
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}

impl<'s, T: Component> QueryItemMut<'s> for &'s mut T {
    type Slice = &'s mut [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: Mask::zero(),
            validation_unique: mask::<T>(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype.components_mut::<T>().unwrap().as_mut_slice()
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &mut slice[index]
    }
}



impl<'s, T: Component> QueryItemMut<'s> for Option<&'s T> {
    type Slice = Option<&'s [T]>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype
            .components::<T>()
            .map(|col| col.as_slice()) 
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        slice.map(|slice| &slice[index])
    }
}

impl<'s, T: Component> QueryItemMut<'s> for Option<&'s mut T> {
    type Slice = Option<&'s mut [T]>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype
            .components_mut::<T>()
            .map(|col| col.as_mut_slice()) 
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        slice.map(|slice| &mut slice[index])
    }
}


impl<'s> QueryItemMut<'s> for &'s Entity {
    type Slice = &'s [Entity];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype.entities()
    }

    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}


impl<'s> QueryItemMut<'s> for &'s () {
    type Slice = &'s [()];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        &[]
    }

    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &()
    }
}