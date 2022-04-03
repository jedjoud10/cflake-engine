use crate::{registry, Archetype, Component, EcsManager, Entity, LayoutQuery, Mask, QueryError};
use std::{
    cell::{RefCell, UnsafeCell},
    marker::PhantomData,
};

// Helps us get queries from archetypes
pub struct QueryBuilder<'a, Layout: LayoutQuery> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // Internal entry mask (calculated from the Layout)
    pub(super) mask: Mask,

    // The queries that are currently being mutably borrowed
    pub(super) accessed: RefCell<Mask>,

    _phantom: PhantomData<Layout>,
}

impl<'a, Layout: LayoutQuery> QueryBuilder<'a, Layout> {
    // Create a new query builder from a layout 
    pub fn new(manager: &'a mut EcsManager) -> Self {
        Self::new_from_mask(manager, Layout::mask())
    }
    // Create a query builder directly from a combined mask
    pub fn new_from_mask(manager: &'a mut EcsManager, mask: Mask) -> Self {
        Self {
            manager,
            mask,
            accessed: Default::default(),
            _phantom: Default::default(),
        }
    }

    // Query
    pub fn query(mut self) -> Vec<Layout::Layout> {
        todo!()
    }
    /*
    pub fn get<T: Component>(&self) -> Result<Vec<&T>, QueryError> {
        let refs = self.get_vec_mapped::<T, &T, _>(|cell| unsafe { &*cell.get() })?;
        Ok(refs)
    }
    pub fn get_mut<T: Component>(&self) -> Result<Vec<&mut T>, QueryError> {
        let mut_refs = self.get_vec_mapped::<T, &mut T, _>(|cell| unsafe { &mut *cell.get() })?;
        Ok(mut_refs)
    }
    */
    /* #endregion */

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
    pub(crate) fn filter_archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.manager.archetypes.iter().filter(move |archetype| self.mask & archetype.mask() == self.mask)
    }
    // Get a vector full of component references (either & or &mut) by running a function through each UnsafeCell
    pub fn get_component_vec_mapped<T: Component, Res, F: FnMut(&UnsafeCell<T>) -> Res + Copy + 'static>(&self, f: F) -> Result<impl Iterator<Item = Res> + '_, QueryError> {
        let component = self.get_component_mask::<T>()?;
        // The component was borrowed, we cannot access it again
        let mut accessed = self.accessed.borrow_mut();
        *accessed = *accessed | component;
        let flatmap = self.filter_archetypes().flat_map(move |archetype| {
            // Fetch the components
            let vec = archetype.vectors().get(&component).unwrap();
            let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
            vec.iter().map(f)
        });

        Ok(flatmap)    
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
