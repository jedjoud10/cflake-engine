use std::any::Any;

use crate::{EcsManager, Mask, Entity, Component, LinkModifierError, component_mask, EntityLinkings, registry, register_unique, Archetype};


// An link modifier that can add additional components to an entity or remove components
pub struct LinkModifier<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    new_components: Vec<(Mask, Box<dyn Any>)>,

    // Bits of the components that were successfully added
    added: Mask,

    // Bits of the components that must be removed
    removed: Mask,

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
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        Self {
            manager,
            new_components: Default::default(),
            added: Default::default(),
            removed: Default::default(),
            entity,
        }
    }
    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkModifierError> {
        // Bits
        let mask = component_mask::<T>()?;
        let new = self.added | mask;

        // Check for link duplication
        if self.added == new {
            return Err(LinkModifierError::LinkDuplication(registry::name::<T>()));
        }

        // Always make sure there is a unique vector for this component
        register_unique::<T>(self.manager, mask);

        // No link duplication. However, there is a chance we removed this component in an earlier call, so we must check for that as well
        // If we did remove it earlier, just overwrite it
        if self.removed & mask != Mask::default() {
            // We did remove it, so simply overwrite it (only if we are part of a valid archetype)
            if let Some(mut entry) = self.manager.entry(self.entity) {
                // Overwrite
                let elem = entry.get_mut::<T>().unwrap();
                *elem = component;

                // Exit early
                return Ok(());
            } else { /* We are not part of an archetype, so add the component normally */ }

            // Remove the component bits from the "removed" mask
            self.removed = self.removed & !mask;
        }

        // Finish it off
        self.added = new;

        // Temporarily store the components
        self.new_components.push((mask, Box::new(component)));
        Ok(())
    }
    // Remove a component from the entity
    pub fn remove<T: Component>(&mut self) -> Result<(), LinkModifierError> {
        // Bits
        let mask = component_mask::<T>()?;

        // Check if we have the component locally stored in this link modifier
        if self.added & mask != Mask::default() {
            // Search for the local component, and remove it
            self.new_components.retain(|(m, _)| *m != mask);
        } else {
            // Removal bits
            self.removed = self.removed | mask;
        }
            
        Ok(())
    }
    // Makes sure an archetype exists
    fn archetype_insert_or_default(&mut self, mask: Mask) -> &mut Archetype {
        // Make sure the target archetype exists
        self.manager.archetypes.entry(mask).or_insert_with(|| {
            // Insert a new archetype
            Archetype::new(mask, &self.manager.uniques)
        })
    }
    // Apply the modifier
    // This will register a new archetype if needed, and it will move the entity from it's old archetype to the new one
    pub(crate) fn apply(mut self, linkings: &mut Option<EntityLinkings>) {
        if let Some(linkings) = linkings {
            // The entity is currently part of an archetype
            let accumulated = linkings.mask | self.added;
            // Get the current archetype
            let current = self.manager.archetypes.get_mut(&linkings.mask).unwrap();

            let target = self.archetype_insert_or_default(accumulated);
            let current = 
            // Move
        } else {
            // First time linkings
            let target = self.archetype_insert_or_default(self.added);
        }
        
        /*

        // Insert the components into the archetype
        let linkings = self
            .manager
            .entities
            .get_mut(self.entity)
            .unwrap()
            .get_or_insert_with(|| EntityLinkings {
                bundle: 0,
                mask: Mask::default(),
            });
        archetype.insert_with(self.new_components, linkings, self.entity);
        */
    }
}
