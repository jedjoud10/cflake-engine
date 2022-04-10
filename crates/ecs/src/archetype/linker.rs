use super::LinkError;
use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    manager::EcsManager,
    Mask, Archetype, EntitySet,
};
use std::any::Any;

// Get the mask of a specific component
pub fn component_mask<T: Component>() -> Result<Mask, LinkError> {
    registry::mask::<T>().map_err(LinkError::ComponentError)
}

// Make sure there is an emtpy unique component vector at our disposal
pub fn register_unique<T: Component>(manager: &mut EcsManager, mask: Mask) {
    // Create a new unique component storage if it is missing
    manager.uniques.entry(mask).or_insert_with(|| Box::new(Vec::<T>::new()));
}

// Linker trait that will be implemented for SimpleLinker and StrictLinker
pub trait Linker<'a> {    
    // Yk, linker shit
    type Input: 'a;
    fn new(input: Self::Input) -> Self;
    fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError>;
    fn apply(self) -> EntityLinkings;    
}

// Component linker that will simply link components to an entity
pub struct SimpleLinker<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    new_components: Vec<(Mask, Box<dyn Any>)>,

    // Bits of the components that were successfully added
    mask: Mask,

    // Entity
    entity: Entity,
}

impl<'a> Linker<'a> for SimpleLinker<'a> {
    // Create a new linker
    fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        type Input;
        Self {
            manager,
            new_components: Default::default(),
            mask: Default::default(),
            entity,
        }
    }
    // Insert a component into the linker (internally), thus linking it to the entity
    fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError> {
        // Bits
        let mask = component_mask::<T>()?;
        let new = self.mask | mask;

        // Check for link duplication
        if self.mask == new {
            return Err(LinkError::LinkDuplication(registry::name::<T>()));
        } else {
            // No link duplication, we can apply the new mask
            self.mask = new;
        }

        // Always make sure there is a unique vector for this component
        register_unique::<T>(self.manager, mask);

        // Temporarily store the components
        self.new_components.push((mask, Box::new(component)));

        // Create a new unique component storage if it is missing
        self.manager.uniques.entry(mask).or_insert_with(|| Box::new(Vec::<T>::new()));
        Ok(())
    }
    // Apply the linker
    fn apply(self) -> EntityLinkings {
        // Make sure the archetype exists
        let archetype = self.manager.archetypes.insert_default(self.mask, &self.manager.uniques);

        // Insert the components into the archetype
        let linkings = self.manager.entities.get_mut(self.entity).unwrap();
        archetype.insert_boxed(self.new_components, linkings, self.entity);
        *linkings
    }

}

// A linker that knows what the target archetype before hand
pub struct StrictLinker<'a> {
    // Archetype and linkings
    archetype: &'a mut Archetype,
    linkings: &'a mut EntityLinkings,

    // Bits of the components that were successfully added
    mask: Mask,
    
    // Entity
    entity: Entity,
}

impl<'a> Linker<'a> for StrictLinker<'a> {
    // Create a new exact linker
    fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        let linkings = manager.entities.get_mut(entity).unwrap();
        let archetype = manager.archetypes.get_mut(linkings.mask).unwrap();
        Self {
            archetype: manager.archetypes.get_mut(),
            linkings,
            mask: Default::default(),
            entity,
        }
    }
    // Insert a component into the target archetype directly
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError> {
        // Bits
        let mask = component_mask::<T>()?;
        let new = self.mask | mask;

        // Return an error if we try to add a component that doesn't belong to our archetype
        if mask & self.archetype.mask == Mask::default() {
            return Err(LinkError::StrictLinkOutlier(registry::name::<T>()))
        }

        // Check for link duplication
        if self.mask == new {
            return Err(LinkError::LinkDuplication(registry::name::<T>()));
        } else {
            // No link duplication, we can apply the new mask
            self.mask = new;
        }

        // Push the component into the target archetype 
        self.archetype.insert_component::<T>(component).map_err(LinkError::ComponentError)?;

        Ok(())
    }
    // Apply the strict linker
    pub(crate) fn apply(self) -> EntityLinkings {
        // Just update the linkings
        self.archetype.push_entity(self.linkings, self.entity);
        *self.linkings
    }
}