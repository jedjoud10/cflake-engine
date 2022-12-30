use std::mem::MaybeUninit;

use crate::{
    mask, Archetype, Component, Mask, MaskHashMap, Column, UntypedColumn,
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
    fn push<'a>(self, storages: &mut Self::Storages<'a>);

    // Get the default tables for this owned bundle
    fn default_tables() -> MaskHashMap<Box<dyn UntypedColumn>>;
}
// Implement the owned bundle for single component
impl<T: Component> Bundle for T {
    type Storages<'a> = &'a mut Vec<T>;

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
        archetype.column_mut::<T>().map(|c| c.components_mut())
    }

    fn push<'a>(self, storages: &mut Self::Storages<'a>) {
        storages.push(self)
    }

    fn default_tables() -> MaskHashMap<Box<dyn UntypedColumn>> {
        let column = Column::<T>::new();
        let boxed: Box<dyn UntypedColumn> = Box::new(column);
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }
}