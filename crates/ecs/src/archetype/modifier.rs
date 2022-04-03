use std::any::Any;

use crate::{component_mask, register_unique, registry, Component, EcsManager, Entity, EntityLinkings, LinkModifierError, Mask};

// An link modifier that can add additional components to an entity or remove components
pub struct LinkModifier<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    new_components: Vec<(Mask, Box<dyn Any>)>,

    // Linkings
    linkings: Option<EntityLinkings>,

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
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        // Fetch the entity's linking mask
        let linkings = manager.entities.get(entity).and_then(|x| x.as_ref());
        let mask = linkings.map(|linkings| linkings.mask).unwrap_or_default();

        Self {
            linkings: linkings.cloned(),
            modified: mask,
            manager,
            new_components: Default::default(),
            entity,
        }
    }
    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkModifierError> {
        // Bits
        let mask = component_mask::<T>()?;
        let new = self.modified | mask;

        // Check for link duplication
        if self.modified == new {
            return Err(LinkModifierError::LinkDuplication(registry::name::<T>()));
        }

        // Always make sure there is a unique vector for this component
        register_unique::<T>(self.manager, mask);

        // Finish it off
        self.modified = new;

        // Check if we can simply overwrite the data
        if let Some(linkings) = self.linkings {
            if linkings.mask & mask != Mask::default() {
                // The current archetype contains components of this type, so we can simply overwrite
                //let mut entry = self.manager.entry(self.entity).unwrap();
                //*entry.get_mut::<T>().unwrap() = component;
                return Ok(());
            } else { /* Add the component normally */
            }
        } else { /* Add the component normally */
        }

        // Temporarily store the components
        self.new_components.push((mask, Box::new(component)));

        Ok(())
    }
    // Remove a component from the entity
    pub fn remove<T: Component>(&mut self) -> Result<(), LinkModifierError> {
        // Bits
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
    pub(crate) fn apply(self, linkings: &mut Option<EntityLinkings>) {
        if let Some(linkings) = linkings {
            // The entity is currently part of an archetype
            let old = linkings.mask;
            let new = self.modified;

            // Check if we even modified the entity
            if new != old {
                // Make sure the target archetype is valid
                self.manager.archetypes.insert_default(new, &self.manager.uniques);

                // Get the current archetype along the target archetype, then move the entity
                dbg!(old);
                dbg!(new);
                let (current, target) = self.manager.archetypes.get_two_mut(old, new).unwrap();
                println!("Moved entity from {} to {}", old, new);
                current.move_entity(self.entity, linkings, self.new_components, target);
            }
        } else {
            // First time linkings, make sure the target archetype is valid
            let archetype = self.manager.archetypes.insert_default(self.modified, &self.manager.uniques);

            // Validate the linkings, then insert the entity into the archetype
            let linkings = linkings.get_or_insert(EntityLinkings { bundle: 0, mask: self.modified });
            archetype.insert_with(self.new_components, linkings, self.entity);
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
        /*


        // No link duplication. However, there is a chance we removed this component in an earlier call, so we must check for that as well
        // If we did remove it earlier, just overwrite it
        if self.removed & mask != Mask::default() {
            // Remove the component bits from the "removed" mask
            self.removed = self.removed & !mask;

            // We did remove it, so simply overwrite it (only if we are part of a valid archetype)
            if let Some(mut entry) = self.manager.entry(self.entity) {
                // Overwrite
                let elem = entry.get_mut::<T>().unwrap();
                *elem = component;

                // Exit early
                return Ok(());
            } else { /* We are not part of an archetype, so add the component normally */ }
        }
        */
    }
}
