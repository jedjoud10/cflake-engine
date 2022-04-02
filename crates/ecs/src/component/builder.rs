use crate::{registry, Archetype, Component, EcsManager, Entity, Mask, QueryError, EntityState};
use std::cell::{RefCell, UnsafeCell};

// Helps us get queries from archetypes
pub struct QueryBuilder<'a> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // Internal entry mask
    pub(super) mask: Mask,

    // The queries that are currently being mutably borrowed
    pub(super) accessed: RefCell<Mask>,
}

impl<'a> QueryBuilder<'a> {
    // Create self from the Ecs manager and some masks
    pub fn new(manager: &'a mut EcsManager, mask: Mask) -> Self {
        Self {
            manager,
            mask,
            accessed: Default::default(),
        }
    }

    /* #region Queries */
    pub fn get<T: Component>(&self) -> Result<Vec<&T>, QueryError> {
        let refs = self.get_vec_mapped::<T, &T, _>(|cell| unsafe { &*cell.get() })?;
        Ok(refs)
    }
    pub fn get_mut<T: Component>(&self) -> Result<Vec<&mut T>, QueryError> {
        let mut_refs = self.get_vec_mapped::<T, &mut T, _>(|cell| unsafe { &mut *cell.get() })?;
        Ok(mut_refs)
    }
    pub fn get_entities(&self) -> Result<Vec<Entity>, QueryError> {
        Ok(self.filter_archetypes().flat_map(|archetype| archetype.entities()).cloned().collect::<Vec<_>>())
    }
    pub fn get_states<T: ContainsState>(&self) -> Result<Vec<T::Item>, QueryError> {
        T::get_states(self)
    }
    /* #endregion */
    
    /* #region Helper functions */
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
    fn filter_archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.manager.archetypes.iter().filter(move |archetype| self.mask & archetype.mask() == self.mask)
    }
    pub fn get_vec_mapped<T: Component, Res, F: FnMut(&UnsafeCell<T>) -> Res + Copy>(&self, f: F) -> Result<Vec<Res>, QueryError> {
        // Combined results
        let mut results = Vec::<Res>::new();

        let component = self.get_component_mask::<T>()?;
        for archetype in self.filter_archetypes() {
            // Fetch the components
            let vec = archetype.vectors().get(&component).unwrap();
            let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
            results.extend(vec.iter().map(f))
        }

        // The component was borrowed, we cannot access it again
        let mut accessed = self.accessed.borrow_mut();
        *accessed = *accessed | component;

        Ok(results)
    }
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

// A trait that will be implemented on structs that can get their states from an archetype bundle.
// More notably: The "Entity" struct and all components that implement the Component trait
pub trait ContainsState {
    type Item: Clone + Copy;
    // Get states from a single archetype 
    fn get_states(builder: &QueryBuilder) -> Result<Vec<Self::Item>, QueryError>;
}

// Components have their "mutation" state
impl<T: Component> ContainsState for T {
    // The state of the component (check if it was written)
    type Item = bool;

    // Get the component states
    fn get_states(builder: &QueryBuilder) -> Result<Vec<Self::Item>, QueryError> {
        // Get the component mask
        let mask = builder.get_component_mask::<T>()?;
        builder.filter_archetypes().flat_map(|archetype| {
            let bitfield = archetype.states().components[&mask];
            bitfield.
        }).cloned().collect::<Vec<_>>()
    }
}