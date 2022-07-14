use crate::{
    entity::{Entity, EntityLinkings},
    registry, ArchetypeSet, EntitySet, Mask, MaskMap, StateRow, States, StorageVec,
    UniqueStoragesSet,
};
use std::any::Any;

// TODO: Comment
pub struct Archetype {
    mask: Mask,
    vectors: MaskMap<Box<dyn StorageVec>>,
    states: States,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create new a archetype based on it's main mask
    pub(crate) fn new(mask: Mask, uniques: &UniqueStoragesSet) -> Self {
        // We must decompose the combined mask into the individual masks and create the storages from that
        let vectors = (0..(registry::count() as usize))
            .into_iter()
            .filter_map(|i| {
                // Make sure the bit is valid
                if (mask >> i) & Mask::one() != Mask::one() {
                    return None;
                }

                // Create the archetype storage
                let mask = Mask::one() << i;

                Some((mask, uniques[&mask].clone_unique_storage()))
            })
            .collect::<_>();

        Self {
            vectors,
            mask,
            entities: Default::default(),
            states: Default::default(),
        }
    }

    // Add an entity into the archetype and update it's linkings
    pub(crate) fn push(
        &mut self,
        entity: Entity,
        linkings: &mut EntityLinkings,
        components: Vec<(Mask, Box<dyn Any>)>,
    ) {
        // Add the entity and update it's linkings
        self.states.push(StateRow::new(self.mask));
        self.entities.push(entity);
        linkings.bundle = self.len() - 1;
        linkings.mask = self.mask;

        // Add the components using their specific storages
        for (mask, component) in components {
            self.fetch_update(mask, |vec| vec.push(component));
        }
    }

    // Update a single underlying storage
    fn fetch_update(
        &mut self,
        mask: Mask,
        function: impl FnOnce(&mut Box<dyn StorageVec>),
    ) -> Option<()> {
        let vec = self.vectors.get_mut(&mask)?;
        function(vec);
        Some(())
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
    pub fn storage(&self) -> &MaskMap<Box<dyn StorageVec>> {
        &self.vectors
    }

    // Get the raw boxed storage vectors mutable
    pub fn storage_mut(&mut self) -> &mut MaskMap<Box<dyn StorageVec>> {
        &mut self.vectors
    }

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
}
