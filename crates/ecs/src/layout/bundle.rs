use crate::{Archetype, ComponentTable, Mask, MaskHashMap, Component, mask};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedBundle<'a>
where
    Self: Sized,
{
    type Storages: 'a;
    fn combined() -> Mask;
    fn is_valid() -> bool;
    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages>;
    fn push(storages: &mut Self::Storages, bundle: Self);
    fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>>;
    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentTable>>,
        index: usize,
    ) -> Option<Self>;
}

// Same as owned bundle, but simply a wrapper to eliminate the 'a lifetime
pub trait Bundle: for<'a> OwnedBundle<'a> {}


// Implement the owned bundle for single component
impl<'a, T: Component> OwnedBundle<'a> for T {
    type Storages = &'a mut Vec<T>;

    fn combined() -> Mask {
        mask::<T>()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages> {
        archetype.table_mut::<T>()
    }

    fn push(storages: &mut Self::Storages, bundle: Self) {
        storages.push(bundle)
    }

    fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>> {
        let boxed: Box<dyn ComponentTable> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentTable>>,
        index: usize,
    ) -> Option<Self> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Some(vec.swap_remove(index))
    }
}

impl<T: Component> Bundle for T {}