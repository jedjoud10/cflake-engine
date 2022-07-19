use crate::{Archetype, ComponentTable, Mask, MaskMap};

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
    fn default_tables() -> MaskMap<Box<dyn ComponentTable>>;
    fn try_swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize)
        -> Option<Self>;
}

// Same as owned bundle, but simply a wrapper to eliminate the 'a lifetime
pub trait Bundle: for<'a> OwnedBundle<'a> {}
