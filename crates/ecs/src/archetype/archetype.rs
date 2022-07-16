use crate::{
    entity::{Entity, EntityLinkings},
    registry, ArchetypeSet, EntitySet, Mask, MaskMap, StateRow, States, ComponentStorage,
    UniqueStoragesSet, Component, mask, OwnedComponentLayout,
};
use std::any::Any;

// TODO: Comment
pub struct Archetype {
    mask: Mask,
    storages: MaskMap<Box<dyn ComponentStorage>>,
    states: States,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create an empty archetype that contains no storage vectors
    pub(crate) fn new(mask: Mask) -> Self {
        Self { mask, storages: Default::default(), states: States::default(), entities: Vec::new() }
    }
    
    // Add multiple entities into the archetype with their corresponding owned components
    // The layout mask for "O" must be equal to the layout mask that this archetype contains
    pub(crate) fn extend_from_slice<O: for<'a> OwnedComponentLayout<'a>>(
        &mut self,
        entities: Vec<(Entity, &mut EntityLinkings)>,
        components: Vec<O>
    ) {
        assert_eq!(entities.len(), components.len());
        assert_eq!(O::mask(), self.mask);

        self.reserve(entities.len());

        for (entity, linkings) in entities {
            self.states.push(StateRow::new(self.mask));
            self.entities.push(entity);
            linkings.bundle = self.len() - 1;
            linkings.mask = self.mask;
        }
        
        let mut storages = O::storages_mut(self);

        for set in components {
            O::insert(set, &mut storages);
        }
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.states.reserve(additional);

        for (_, storage) in &mut self.storages {
            storage.reserve(additional);
        }
    }

    // Get the number of entities that reference this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    // Get a list of the entities that are stored within this archetype
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // Get the unique archetype mask
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the current component states immutably
    pub fn states(&self) -> &States {
        &self.states
    }

    // Get the raw boxed storage vectors immutably
    pub fn boxed_storage(&self) -> &MaskMap<Box<dyn ComponentStorage>> {
        &self.storages
    }

    // Get the raw boxed storage vectors mutable
    pub fn boxed_storage_mut(&mut self) -> &mut MaskMap<Box<dyn ComponentStorage>> {
        &mut self.storages
    }

    // Try to get an immutable reference to the storage for a specific component
    pub fn storage<T: Component>(&self) -> Option<&Vec<T>> {
        let boxed = self.storages.get(&mask::<T>())?;
        Some(boxed.as_any().downcast_ref().unwrap())
    }
    
    // Try to get a mutable reference to the storage for a specific component
    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        let boxed = self.storages.get_mut(&mask::<T>())?;
        Some(boxed.as_any_mut().downcast_mut().unwrap())
    }

    /*
    // Remove an entity from the archetype it is currently linked to
    // This will return the removed boxed components that validate the given mask
    pub(crate) fn remove(
        archetypes: &mut ArchetypeSet,
        entities: &mut EntitySet,
        entity: Entity,
        filter: Mask,
    ) -> Vec<(Mask, Box<dyn Any>)> {
        // Get the archetype directly
        let linkings = entities.get_mut(entity).unwrap();
        let bundle = linkings.bundle;
        let archetype = archetypes.get_mut(&linkings.mask).unwrap();

        // The boxed components that will be added into the new archetype
        let mut components: Vec<(Mask, Box<dyn Any>)> =
            Vec::with_capacity(filter.count_ones() as usize);

        // Remove the components from the storages
        for (&mask, vec) in archetype.vectors.iter_mut() {
            // Filter the components that validate the mask
            if mask & filter == mask {
                // Remove the component, and box it
                components.push((mask, vec.swap_remove_boxed(bundle)));
            } else {
                // Remove it normally
                vec.swap_remove(bundle);
            }
        }

        // Remove the entity and get the entity that was swapped with it
        archetype.entities.swap_remove(bundle);
        archetype.states.swap_remove(bundle);
        let entity = archetype.entities.get(bundle).cloned();

        // Swap is not nessecary when removeing the last element anyways
        if let Some(entity) = entity {
            // Since the last entity stored will swap positions, we must update it's linkings
            let swapped_linkings = entities.get_mut(entity).unwrap();
            swapped_linkings.bundle = bundle;
        }

        components
    }
    */
    /*
    // Move an entity from an archetype to another archetype, whilst adding extra components to the entity
    pub(crate) fn move_entity(
        archetypes: &mut ArchetypeSet,
        entities: &mut EntitySet,
        old: Mask,
        new: Mask,
        entity: Entity,
        linkings: &mut EntityLinkings,
        extra: Vec<(Mask, Box<dyn Any>)>,
    ) {
        // Remove the entity (this might fail in the case of the default empty archetype)
        let mut removed = (old != Mask::zero())
            .then(|| Archetype::remove(archetypes, entities, entity, old))
            .unwrap_or_default();

        // Combine the removed components with the extra components
        removed.extend(extra);

        // And insert into the new archetype
        let new = archetypes.get_mut(&new).unwrap();
        new.push(entity, linkings, removed);
    }
    */
}
