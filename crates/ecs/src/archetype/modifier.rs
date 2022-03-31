use super::{Archetype, LinkModifierError};
use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    manager::EcsManager,
    Mask,
};
use std::{any::Any, cell::UnsafeCell};

// Get the mask of a specific component
fn component_mask<T: Component>() -> Result<Mask, LinkModifierError> {
    registry::mask::<T>().map_err(LinkModifierError::ComponentError)
}

// Make sure there is an emtpy unique component vector at our disposal
fn register_unique<T: Component>(manager: &mut EcsManager, mask: Mask) {
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
            } else { /* We are not part of an archetype, so add the component normally */
            }

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
        }

        // Removal bits
        self.removed = self.removed | mask;

        Ok(())
    }
    // Apply the linker
    pub(crate) fn apply(self, _linkings: &mut Option<EntityLinkings>) {
        /*
        // Make sure the archetype exists
        let archetype = self
            .manager
            .archetypes
            .entry(self.combined)
            .or_insert_with(|| {
                // Insert a new archetype
                Archetype::new((&self.masks, self.combined), &self.manager.uniques)
            });

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
