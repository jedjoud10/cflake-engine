use crate::{
    mask, Archetype, Component, ComponentTable, LayoutAccess, LinkError, Mask, MaskMap,
};

// An owned layout trait will be implemented for owned tuples that contain a set of components
pub trait OwnedBundle<'a>
where
    Self: Sized,
{
    // Mutable references to the required vectors stored within the archetypes
    type Storages: 'a;

    // Get the combined mask of the owned layout
    fn combined() -> Mask;

    // Check if this bundle is valid (a bundle is invalid if it has intersecting components)
    fn is_valid() -> bool;

    // Fetch the necessary storages from the archetype
    fn fetch(archetype: &'a mut Archetype) -> Self::Storages;

    // Push a new bundle into the storages
    fn push(storages: &mut Self::Storages, bundle: Self);
}

// Internal owned bundle that we will only use to create archetypes and their storeages
pub trait OwnedBundleTableAccessor: for<'a> OwnedBundle<'a> {
    // Get the default component tables that correspond to this bundle
    fn default_tables() -> MaskMap<Box<dyn ComponentTable>>;
    
    // Steal the underlying bundle from the given component tables
    fn swap_remove(tables: &mut MaskMap<Box<dyn ComponentTable>>, index: usize) -> Self;
    
    // Insert a new bundle into the given component tables
    fn push(storages: &mut MaskMap<Box<dyn ComponentTable>>, bundle: Self) -> Self;
}