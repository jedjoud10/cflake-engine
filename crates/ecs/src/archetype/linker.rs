use super::LinkError;
use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    manager::EcsManager,
    Archetype, Mask,
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

// Either a simple linker or strict linker
enum InternalLinker<'a> {
    Simple {
        manager: &'a mut EcsManager,
        new_components: Vec<(Mask, Box<dyn Any>)>,
    },
    Strict {
        archetype: &'a mut Archetype,
        linkings: &'a mut EntityLinkings,
    },
}

// Component linker that will simply link components to an entity
pub struct Linker<'a> {
    internal: InternalLinker<'a>,

    // Bits of the components that were successfully added
    mask: Mask,

    // Entity
    entity: Entity,    
}

impl<'a> Linker<'a> {
    // Create a new simple linker
    pub(crate) fn new_simple(manager: &'a mut EcsManager, entity: Entity) -> Self {
        Self {
            internal: InternalLinker::Simple {
                manager,
                new_components: Default::default(),
            },
            mask: Default::default(),
            entity,
        }
    }
    // Create a new strict linker
    pub(crate) fn new_strict(entity: Entity, archetype: &'a mut Archetype, linkings: &'a mut EntityLinkings) -> Self {
        Self {
            internal: InternalLinker::Strict { archetype, linkings, },
            mask: Default::default(),
            entity,
        }
    }
    // Insert a component into the linker (internally), thus linking it to the entity
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), LinkError> {
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

        match &mut self.internal {
            InternalLinker::Simple { manager, new_components } => {
                let i = std::time::Instant::now();
                // Always make sure there is a unique vector for this component
                register_unique::<T>(*manager, mask);

                // Temporarily store the components
                new_components.push((mask, Box::new(component)));
                //dbg!(i.elapsed());
            }
            InternalLinker::Strict { archetype, linkings } => {
                // Return an error if we try to add a component that doesn't belong to our archetype
                if mask & archetype.mask == Mask::default() {
                    return Err(LinkError::StrictLinkInvalid(registry::name::<T>()));
                }

                // Insert the component directly into the arhcetype
                archetype.insert_component(component).map_err(LinkError::ComponentError)?;

                // And do a bit of trolling
                linkings.mask = new;
            }
        }

        Ok(())
    }
    // Apply the linker
    pub(crate) fn apply(self) -> EntityLinkings {
        match self.internal {
            InternalLinker::Simple { manager, new_components } => {
                // Make sure the archetype exists
                let archetype = manager.archetypes.entry(self.mask).or_insert_with(|| Archetype::new(self.mask, &manager.uniques));

                // Insert the components into the archetype
                let linkings = manager.entities.get_mut(self.entity).unwrap();
                archetype.insert_boxed(new_components, linkings, self.entity);
                *linkings
            }
            InternalLinker::Strict { archetype, linkings } => {
                // Handle component mismatch
                if archetype.mask != linkings.mask {
                    panic!("Cannot insert entity batch that contains different component layouts. Mismatched layout: {:?}, archetype-layout: {:?}", linkings.mask, archetype.mask);
                }

                // Just in case
                assert_eq!(linkings.mask, self.mask);

                // Otherwise, insert the entity normally
                archetype.push_entity(linkings, self.entity);
                *linkings
            }
        }
    }
}
