use crate::{
    registry, Archetype, Component, EcsManager, Entity, EntityLinkings, LinkError,
    Mask,
};
use std::any::Any;

// A link modifier that will either link or remove components from an entity
pub struct LinkModifier<'a> {
    manager: &'a mut EcsManager,
    locals: Vec<(Mask, Box<dyn Any>)>,
    old: Mask,
    new: Mask,
    entity: Entity,
}

impl<'a> LinkModifier<'a> {
    // 1) Remove T
    // 2) Insert T
    // Overwrite T

    // 1) Insert T
    // 2) Remove T
    // Nothing

    // Create a new link modifier
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Fetch the entity's linking mask
        let linkings = *manager.entities.get(entity)?;

        Some(Self {
            old: linkings.mask,
            new: linkings.mask,
            manager,
            locals: Default::default(),
            entity,
        })
    }

    // Insert a component into the modifier, thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError> {
        let mask = registry::mask::<T>();

        // Always make sure there is a unique vector for this component
        self.manager
            .uniques
            .entry(mask)
            .or_insert_with(|| Box::new(Vec::<T>::new()));

        // The component might've been removed, and if it was, we must cancel it out
        if self.old & mask == mask {
            // Cancel the removal, and overwrite the internally stored component
            let (_, boxed) = self.locals.iter_mut().find(|(m, _)| *m == mask).unwrap();
            *boxed.downcast_mut::<T>().unwrap() = component;
        } else {
            // Check if the current archetype contains component of this type
            if self.old & mask == Mask::zero() {
                // Add the component normally
                self.locals.push((mask, Box::new(component)));
            } else {
                // Overwrite the component
                let mut entry = self.manager.mut_entry(self.entity).unwrap();
                *entry.get_mut::<T>().unwrap() = component;
            }
        }
        self.new = self.new | mask;

        Ok(())
    }

    // Remove a component from the entity
    pub fn remove<T: Component>(&mut self) -> Result<(), LinkError> {
        let mask = registry::mask::<T>();
        let linked_to_entity = self.old & mask == mask;
        let locally_stored = self.new & mask != Mask::zero();

        // Check if we even have the component in the first place
        if !linked_to_entity && !locally_stored {
            return Err(LinkError::ComponentMissing(registry::name::<T>()));
        }

        // Check if we have the component locally stored
        if !linked_to_entity && locally_stored {
            // Search for the local component, and remove it
            self.locals.retain(|(m, _)| *m != mask);
        }

        // Remove the bits
        self.new = self.new & !mask;

        Ok(())
    }

    // This will register a new archetype if needed, and it will move the entity from it's old archetype to the new one
    pub(crate) fn apply(self, linkings: &mut EntityLinkings) {
        // Check if we even modified the entity
        if self.new != self.old {
            // Make sure the target archetype is valid
            self.manager.archetypes
                .entry(self.new)
                .or_insert_with(|| Archetype::new(self.new, &self.manager.uniques));

            // Move the entity to the new archetype
            Archetype::move_entity(
                &mut self.manager.archetypes,
                &mut self.manager.entities,
                self.old,
                self.new,
                self.entity,
                linkings,
                self.locals,
            )
            //println!("Moved entity from {} to {}", self.old, self.new);
        }
    }
}
