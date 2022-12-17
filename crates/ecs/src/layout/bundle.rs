use crate::{
    mask, Archetype, Component, ComponentColumn, Mask, MaskHashMap,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedBundle<'a>
where
    Self: Sized,
{
    type Storages: 'a;

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
    fn prepare(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages>;

    // Push an element into those tables
    fn push(storages: &mut Self::Storages, bundle: Self);

    // Get the default tables for this owned bundle
    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>>;

    // Try to remove and element from the tables, and try to return the cast element
    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        index: usize,
    ) -> Option<Self>;
}

// Same as owned bundle, but simply a wrapper to eliminate the 'a lifetime
pub trait Bundle: for<'a> OwnedBundle<'a> {}
impl<T: for<'a> OwnedBundle<'a>> Bundle for T {}

// Implement the owned bundle for single component
impl<'a, T: Component> OwnedBundle<'a> for T {
    type Storages = &'a mut Vec<T>;

    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(mask::<T>())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages> {
        archetype.components_mut::<T>()
    }

    fn push(storages: &mut Self::Storages, bundle: Self) {
        storages.push(bundle)
    }

    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>> {
        let boxed: Box<dyn ComponentColumn> =
            Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        index: usize,
    ) -> Option<Self> {
        let boxed = tables.get_mut(&mask::<T>())?;
        let vec =
            boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Some(vec.swap_remove(index))
    }
}

// Implement the owned bundle for the unit tuple
impl<'a> OwnedBundle<'a> for () {
    type Storages = ();

    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(Mask::zero())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    fn is_valid() -> bool {
        true
    }

    fn prepare(
        archetype: &'a mut Archetype,
    ) -> Option<Self::Storages> {
        archetype.mask().is_zero().then_some(())
    }

    fn push(_storages: &mut Self::Storages, _bundle: Self) {}

    fn default_tables() -> MaskHashMap<Box<dyn ComponentColumn>> {
        MaskHashMap::default()
    }

    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentColumn>>,
        _index: usize,
    ) -> Option<Self> {
        tables.is_empty().then_some(())
    }
}
