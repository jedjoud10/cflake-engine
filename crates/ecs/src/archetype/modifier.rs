use std::any::Any;

use super::{component_mask, register_archetype, register_unique};
use crate::{registry, Archetype, Component, EcsManager, Entity, EntityLinkings, LinkError, Mask};

// An link modifier that can add additional components to an entity or remove components
pub struct LinkModifier<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    new_components: Vec<(Mask, Box<dyn Any>)>,

    // Linkings
    linkings: EntityLinkings,

    // The modified linking mask of the entity
    modified: Mask,

    // Entity
    entity: Entity,
}

impl<'a> LinkModifier<'a> {
    // 1) Remove T
    // 2) Insert T
    // Overwrite T

    // 1) Insert T
    // 2) Remove T
    // Nothing

    // Create a new extra link modifier
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Fetch the entity's linking mask
        let linkings = *manager.entities.get(entity)?;

        Some(Self {
            linkings,
            modified: linkings.mask,
            manager,
            new_components: Default::default(),
            entity,
        })
    }
    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError> {
        let mask = component_mask::<T>()?;
        let new = self.modified | mask;

        // Check for link duplication
        if self.modified == new {
            return Err(LinkError::LinkDuplication(registry::name::<T>()));
        }

        // Always make sure there is a unique vector for this component
        register_unique::<T>(self.manager, mask);

        // Finish it off
        self.modified = new;

        // Check if we can simply overwrite the data
        if self.linkings.mask & mask != Mask::default() {
            // The current archetype contains components of this type, so we can simply overwrite
            let mut entry = self.manager.entry(self.entity).unwrap();
            *entry.get_mut::<T>().unwrap() = component;
            return Ok(());
        } else { /* Add the component normally */
        }

        // Temporarily store the components
        self.new_components.push((mask, Box::new(component)));

        Ok(())
    }
    // Remove a component from the entity
    pub fn remove<T: Component>(&mut self) -> Result<(), LinkError> {
        let mask = component_mask::<T>()?;

        // Check if we have the component locally stored in this link modifier
        if self.modified & mask != Mask::default() {
            // Search for the local component, and remove it
            self.new_components.retain(|(m, _)| *m != mask);
        }

        // Remove the bits
        self.modified = self.modified & !mask;

        Ok(())
    }
    // Apply the modifier
    // This will register a new archetype if needed, and it will move the entity from it's old archetype to the new one
    // This returns the old mask and new mask
    pub(crate) fn apply(self, linkings: &mut EntityLinkings) -> (Mask, Mask) {
        // The entity is currently part of an archetype
        let old = self.linkings.mask;
        let new = self.modified;

        // Check if we even modified the entity
        if new != old {
            // Make sure the target archetype is valid
            register_archetype(&mut self.manager.archetypes, new, &self.manager.uniques);

            // Get the current archetype along the target archetype, then move the entity
            dbg!(old);
            dbg!(new);
            let (current, target) = self.manager.get_disjoint_archetypes(old, new).unwrap();
            println!("Moved entity from {} to {}", old, new);
            current.move_entity(self.entity, linkings, self.new_components, target);
        }

        (old, new)
    }
}
