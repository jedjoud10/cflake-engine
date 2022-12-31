use std::mem::MaybeUninit;

use crate::{
    mask, Archetype, Component, Mask, MaskHashMap, UntypedColumn, StateFlags, UntypedVec, StateColumn,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
// This will also handle the synchronization between the states/component columns whenever we add bundles
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

    // Get the default untyped component vectors for this bundle
    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>>;
}

trait RemovalBundle {}

// Implement the bundle for single component
impl<T: Component> Bundle for T {
    type Storages<'a> = (&'a mut Vec<T>, &'a mut StateColumn);

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
            storages.0.push(bundle);
            additional += 1;
        }

        storages.1.extend_with_flags(additional, StateFlags {
            added: true,
            modified: true,
        });
        additional
    }

    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>> {
        let boxed: Box<dyn UntypedVec> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }
}