use super::{Archetype, LinkModifierError};
use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    manager::EcsManager,
    Mask,
};
use std::{any::Any, cell::UnsafeCell};

// Get the mask of a specific component
pub fn component_mask<T: Component>() -> Result<Mask, LinkModifierError> {
    registry::mask::<T>().map_err(LinkModifierError::ComponentError)
}

// Make sure there is an emtpy unique component vector at our disposal
pub fn register_unique<T: Component>(manager: &mut EcsManager, mask: Mask) {
    // Create a new unique component storage if it is missing
    manager.uniques.entry(mask).or_insert_with(|| Box::new(Vec::<UnsafeCell<T>>::new()));
}

// Component linker that will simply link components to an entity
pub struct Linker<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    new_components: Vec<(Mask, Box<dyn Any>)>,

    // Bits of the components that were successfully added
    mask: Mask,

    // Entity
    entity: Entity,
}

impl<'a> Linker<'a> {
    // Create a new component linker
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        Self {
            manager,
            new_components: Default::default(),
            mask: Default::default(),
            entity,
        }
    }
    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkModifierError> {
        // Bits
        let mask = component_mask::<T>()?;
        let new = self.mask | mask;

        // Check for link duplication
        if self.mask == new {
            return Err(LinkModifierError::LinkDuplication(registry::name::<T>()));
        } else {
            // No link duplication, we can apply the new mask
            self.mask = new;
        }

        // Temporarily store the components
        self.new_components.push((mask, Box::new(component)));

        // Create a new unique component storage if it is missing
        self.manager.uniques.entry(mask).or_insert_with(|| Box::new(Vec::<UnsafeCell<T>>::new()));
        Ok(())
    }
    // Apply the linker
    pub(crate) fn apply(self) {
        // Make sure the archetype exists
        let archetype = self.manager.archetypes.entry(self.mask).or_insert_with(|| {
            // Insert a new archetype
            Archetype::new(self.mask, &self.manager.uniques)
        });

        // Insert the components into the archetype
        let linkings = self
            .manager
            .entities
            .get_mut(self.entity)
            .unwrap()
            .get_or_insert_with(|| EntityLinkings { bundle: 0, mask: Mask::default() });
        archetype.insert_with(self.new_components, linkings, self.entity);
    }
}
