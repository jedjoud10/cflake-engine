use smallvec::SmallVec;

use crate::{registry, Archetype, Component, EcsManager, Entity, LayoutQuery, Mask, QueryError, ARCHETYPE_INLINE_SIZE, ComponentError};
use std::{
    cell::{RefCell, UnsafeCell},
    marker::PhantomData,
};

// Type aliases
pub type InlinedStorages<'a, T> = SmallVec<[&'a [UnsafeCell<T>]; ARCHETYPE_INLINE_SIZE]>;

// Helps us get queries from archetypes
pub struct Query<'a, Layout: LayoutQuery> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // Internal entry mask (calculated from the Layout)
    pub(super) mask: Mask,

    // The queries that are currently being mutably borrowed
    pub(super) accessed: RefCell<Mask>,

    _phantom: PhantomData<Layout>,
}

impl<'a, Layout: LayoutQuery> Query<'a, Layout> {
    // Create a new query builder from a layout
    pub fn new(manager: &'a mut EcsManager) -> Result<Self, QueryError> {
        Ok(Self {
            manager,
            mask: Layout::mask().map_err(QueryError::ComponentError)?,
            accessed: Default::default(),
            _phantom: Default::default(),
        })
    }

    // Consume the query, returning it's vector
    pub fn consume(self) -> Vec<Layout::Layout> {
        let i = std::time::Instant::now();
        let vec = Layout::query(&self).unwrap();
        dbg!(i.elapsed());
        vec
    }

    /* #region Helper functions */
    // Get a specific component mask. This might fail
    fn get_component_mask<T: Component>(&self) -> Result<Mask, QueryError> {
        // Component mask
        let mask = registry::mask::<T>().map_err(QueryError::ComponentError)?;

        // Check if the component mask is even valid
        if mask & self.mask == Mask::default() {
            return Err(QueryError::Unlinked(registry::name::<T>()));
        }

        // Check if the component is currently mutably borrowed
        if mask & *self.accessed.borrow() != Mask::default() {
            return Err(QueryError::MutablyBorrowed(registry::name::<T>()));
        }

        Ok(mask)
    }
    // Filter the archetypes based on the interally stored mask
    pub(super) fn filter_archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.manager.archetypes.iter().filter(move |archetype| self.mask & archetype.mask() == self.mask)
    }
    // Get a vector that contains all the underlying storages
    pub(super) fn get_storages<T: Component>(&self) -> InlinedStorages<T> {
        let component = self.get_component_mask::<T>().unwrap();
        // The component was borrowed, we cannot access it again
        let mut accessed = self.accessed.borrow_mut();
        *accessed = *accessed | component;
        self.filter_archetypes().map(move |archetype| {
            // Fetch the components
            let vec = archetype.vectors().get(&component).unwrap();
            let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
            vec.as_slice()
        }).collect::<InlinedStorages<T>>()
    }
    // Get a singular pointer to a component in an archetype and at a specific bundleindex
    pub fn get_ptr<T: Component>(&self, bundle: usize, mask: Mask) -> Result<*mut T, QueryError> {
        // Get the component mask
        let component_mask = self.get_component_mask::<T>()?;

        // And then get the singular component
        let archetype = self
            .manager
            .archetypes
            .get(&mask)
            .ok_or_else(|| QueryError::DirectAccessArchetypeMissing(mask, registry::name::<T>()))?;

        // Read from the rwlock
        let storage = archetype.vectors().get(&component_mask).unwrap();

        // Just fetch the pointer
        let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        let component = vec.get(bundle).ok_or_else(|| QueryError::DirectAccessBundleIndexInvalid(bundle, registry::name::<T>()))?;
        Ok(component.get())
    }
    /* #endregion */
}
