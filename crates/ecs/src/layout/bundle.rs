use crate::{Archetype, ComponentTable, Mask, MaskHashMap, Component, mask, BundleError, name};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedBundle<'a>
where
    Self: Sized,
{
    type Storages: 'a;

    // Get the number of components that this bundle contains
    fn items() -> usize;

    // Get the debug name of a specific component
    fn name(index: usize) -> Option<&'static str>;

    // Get the layout accesses of a specific component
    fn mask(index: usize) -> Option<Mask>;
    
    // Get a combined  mask by running a lambda on each mask
    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask;

    // Check if the bundle is valid for usage from a specific arhcetype
    fn is_valid(archetype: &Archetype) -> Result<(), BundleError> {
        let combined = Self::reduce(|a, b| a | b);        

        // Check if we have any missing components from the archetype
        if combined & archetype.mask() != combined  {
            let diff = (combined & archetype.mask()) ^ combined;
            let index = (0..Self::items()).position(|i| {
                let local = Self::mask(i).unwrap();
                local & diff == local
            }).unwrap();
            return Err(BundleError::MissingArchetypeTable(Self::name(index).unwrap()));
        }

        let mask = Self::reduce(|a, b| a | b);
        let converted: u64 = mask.into();
        if converted.count_ones() != Self::items() as u32 {
            let mut accumulator = Mask::zero();
            let index = (0..Self::items()).into_iter().position(|i| {
                let copy = accumulator;
                accumulator = accumulator | Self::mask(i).unwrap();
                copy != accumulator
            }).unwrap();

            Err(BundleError::DuplicateComponent(Self::name(index).unwrap()))
        } else {
            Ok(())
        }
    }

    // Get the storage tables once and for all
    fn prepare(archetype: &'a mut Archetype) -> Option<Self::Storages>;

    // Push an element into those tables
    fn push(storages: &mut Self::Storages, bundle: Self);

    // Get the default tables for this owned bundle
    fn default_tables() -> MaskHashMap<Box<dyn ComponentTable>>;

    // Try to remove and element from the tables, and try to return the cast element
    fn try_swap_remove(
        tables: &mut MaskHashMap<Box<dyn ComponentTable>>,
        index: usize,
    ) -> Option<Self>;
}

// Same as owned bundle, but simply a wrapper to eliminate the 'a lifetime
pub trait Bundle: for<'a> OwnedBundle<'a> {}
impl<T: for<'a> OwnedBundle<'a>> Bundle for T {}


// Implement the owned bundle for single component
impl<'a, T: Component> OwnedBundle<'a> for T {
    type Storages = &'a mut Vec<T>;

    fn items() -> usize {
        1
    }

    fn name(index: usize) -> Option<&'static str> {
        (index == 0).then(|| name::<T>())
    }

    fn mask(index: usize) -> Option<Mask> {
        (index == 0).then(|| mask::<T>())
    }

    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(mask::<T>()).into_iter().reduce(lambda).unwrap()
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