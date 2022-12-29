use std::mem::MaybeUninit;

use crate::{
    mask, Archetype, Component, ComponentColumn, Mask, MaskHashMap, Column,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait Bundle: Sized {
    type Storages<'a>: 'a;

    // Get a combined  mask by running a lambda on each mask
    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask;

    // Checks if this bundle is valid
    fn is_valid() -> bool {
        let mut count = 1;
        let mask = Self::reduce(|a, b| {
            count += 1;
            a | b
        });
        mask.count_ones() == count as u32
    }

    // Get the storage tables once and for all
    fn prepare<'a>(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages<'a>>;

    // Push an element into those tables
    fn push<'a>(storages: &mut Self::Storages<'a>, bundle: Self);

    // Get the default tables for this owned bundle
    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>>;

    // Try to get a default bundle (only used for unit bundles)
    fn try_get_default() -> Option<Self> { None }

    // Try to remove and element from the tables, and try to return the cast element
    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        index: usize,
    ) -> Option<Self>;
}
// Implement the owned bundle for single component
impl<T: Component> Bundle for T {
    type Storages<'a> = &'a mut Column<T>;

    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(mask::<T>())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare<'a>(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages<'a>> {
        archetype.components_mut::<T>()
    }

    fn push<'a>(storages: &mut Self::Storages<'a>, bundle: Self) {
        storages.push(MaybeUninit::new(bundle))
    }

    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>> {
        let boxed: Box<dyn ComponentColumn> = Box::new(Column::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        index: usize,
    ) -> Option<Self> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec =
            boxed.as_any_mut().downcast_mut::<Column<T>>().unwrap();
        Some(unsafe { vec.swap_remove(index).assume_init() })
    }
}

// Implement the owned bundle for the unit tuple
impl Bundle for () {
    type Storages<'a> = ();

    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(Mask::zero())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare<'a>(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages<'a>> {
        archetype.mask().is_zero().then_some(())
    }

    fn push<'a>(_storages: &mut Self::Storages<'a>, _bundle: Self) {}

    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>> {
        MaskHashMap::default()
    }

    fn try_get_default() -> Option<Self> {
        Some(())
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        _index: usize,
    ) -> Option<Self> {
        tables.is_empty().then_some(())
    }
}
