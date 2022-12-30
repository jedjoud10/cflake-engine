use std::mem::MaybeUninit;

use crate::{
    mask, Archetype, Component, Mask, MaskHashMap, Column, UntypedColumn, StateFlags, UntypedVec,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait Bundle: Sized + 'static {
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

    // Push multiple elements into those storages, returns how many we added
    fn extend_from_iter<'a>(
        storages: &mut Self::Storages<'a>,
        iter: impl IntoIterator<Item = Self>
    ) -> usize;

    // Get the default untyped component columns for this bundle
    fn default_columns() -> MaskHashMap<Box<dyn UntypedColumn>>;

    // Get the default untyped component vectors for this bundle
    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>>;
}

trait RemovalBundle {}

// Implement the bundle for single component
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
        archetype.column_mut::<T>()
    }

    fn extend_from_iter<'a>(
        storages: &mut Self::Storages<'a>,
        iter: impl IntoIterator<Item = Self>
    ) -> usize {
        let mut additional = 0;
        for bundle in iter {
            storages.components_mut().push(bundle);
            additional += 1;
        }

        storages.states_mut().extend_with_flags(additional, StateFlags {
            added: true,
            modified: true,
        });
        additional
    }

    fn default_columns() -> MaskHashMap<Box<dyn UntypedColumn>> {
        let boxed: Box<dyn UntypedColumn> = Box::new(Column::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>> {
        let boxed: Box<dyn UntypedVec> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }
}