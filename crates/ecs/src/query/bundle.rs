use crate::{
    mask, Archetype, Component, ComponentTable, LayoutAccess, LinkError, Mask, MaskMap,
};

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
}

// Internal owned bundle that we will only use to create archetypes and their storages
pub trait OwnedBundleAnyTableAccessor: for<'a> OwnedBundle<'a> {
    fn default_tables() -> MaskMap<Box<dyn ComponentTable>>;
    fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Option<Self>;
    fn push(tables: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Option<()>;
}