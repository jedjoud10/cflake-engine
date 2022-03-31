use super::{Archetype, ArchetypeError, LinkModifierError};
use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    manager::EcsManager,
    prelude::Mask,
};
use std::{
    any::{type_name, Any},
    cell::UnsafeCell,
};
use tinyvec::ArrayVec;

// A linkg modifier that will add/remove components to/from an entity
pub struct LinkModifier<'a> {
    // Manager
    manager: &'a mut EcsManager,

    // The stored components
    components: Vec<(Mask, Box<dyn Any>)>,

    // Bits of the components that were successfully added
    combined: Mask,
    masks: ArrayVec<[Mask; 64]>,

    // Entity
    entity: Entity,
}

impl<'a> LinkModifier<'a> {
    // Create a new link modifier
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Self {
        Self {
            manager,
            components: Default::default(),
            combined: Default::default(),
            masks: Default::default(),
            entity,
        }
    }
    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkModifierError> {
        // Bits
        let mask = registry::mask::<T>().map_err(|err| LinkModifierError::ComponentError(err))?;
        let new = self.combined | mask;

        // Check for link duplication
        if self.combined == new {
            return Err(LinkModifierError::LinkDuplication(registry::name::<T>()));
        }
        self.combined = new;
        self.masks.push(mask);

        // Temporarily store the components
        self.components.push((mask, Box::new(component)));

        // Create a new unique component storage if it is missing
        self.manager
            .uniques
            .entry(mask)
            .or_insert_with(|| Box::new(Vec::<UnsafeCell<T>>::new()));
        Ok(())
    }
    // Try to remove a component from the entity
    pub fn remove<T: Component>(&mut self) -> Result<T, LinkModifierError> {
        todo!()
    }
    // Apply the linker
    pub(crate) fn apply(self) {
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
        archetype.insert_with(self.components, linkings, self.entity);
    }
}