use crate::{
    entity::{Entity, EntityLinkings},
    mask, ArchetypeSet, Bundle, Component, EntitySet, Mask, MaskHashMap, PrefabBundle,
    RemovedComponents, StateColumn, UntypedColumn,
};

// The table that will be stored internally
pub type Table = MaskHashMap<UntypedColumn>;

// An archetype is a special structure that contains multiple entities of the same layout
// Archetypes are used in archetypal ECSs to improve iteration and insertion/removal performance
pub struct Archetype {
    mask: Mask,

    table: Table,

    // Entities that have the same mask as the archetype's mask
    entities: Vec<Entity>,
}

impl Archetype {
    // Create a new archetype from a owned bundle accessor
    // This assumes that B is a valid bundle
    pub(crate) fn from_bundle<B: Bundle>() -> Self {
        let mask = B::reduce(|a, b| a | b);

        log::debug!("Creating archetype from bundle of mask {:?}", mask);

        let defaults = B::default_vectors()
            .into_iter()
            .map(|(mask, vec)| (mask, UntypedColumn::new(vec)));

        Self {
            mask,
            table: Table::from_iter(defaults),
            entities: Vec::new(),
        }
    }

    // Create the unit archetype that contains no columns and has a zeroed mask
    pub(crate) fn empty() -> Self {
        Self {
            mask: Mask::zero(),
            table: Default::default(),
            entities: Default::default(),
        }
    }

    pub(crate) fn instantiate_prefab(
        &mut self,
        entities: &mut EntitySet,
        prefab: &Box<dyn PrefabBundle>,
    ) -> Entity {
        let index = self.entities.len();

        let linkings = EntityLinkings {
            mask: self.mask,
            index,
        };
        let entity = entities.insert(linkings);
        self.entities.push(entity);

        prefab.prefabify(self).unwrap();

        log::debug!("Extended archetype {} with instantiated prefab", self.mask,);

        self.entities[index]
    }

    // Add multiple entities into the archetype with their corresponding owned components
    // The layout mask for "B" must be equal to the layout mask that this archetype contains
    // This will also add the entities to the entity set
    pub(crate) fn extend_from_iter<B: Bundle>(
        &mut self,
        entities: &mut EntitySet,
        components: impl IntoIterator<Item = B>,
    ) -> &[Entity] {
        assert_eq!(self.mask(), B::reduce(|a, b| a | b));
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Reserve more memory to reduce the number of reallocation
        let iter = components.into_iter();
        self.reserve(iter.size_hint().0);
        let old_len = self.entities.len();

        // Add the components first (so we know how many entities we need to add)
        let additional = B::extend_from_iter(self, iter).unwrap();

        // Allocate the entities then add them as well
        for i in 0..additional {
            let linkings = EntityLinkings {
                mask: self.mask,
                index: old_len + i,
            };
            let entity = entities.insert(linkings);
            self.entities.push(entity);
        }

        log::debug!(
            "Extended archetype {} with {} new element(s)",
            self.mask,
            additional
        );

        // Return the newly added entity IDs
        &self.entities[old_len..]
    }

    // Remove multiple entities from the archetype and dissociate their components
    // This will also remove the entities from the entity set
    pub(crate) fn remove_from_iter(
        &mut self,
        entities: &mut EntitySet,
        iter: impl Iterator<Item = (Entity, EntityLinkings)>,
        removed: &mut RemovedComponents,
    ) {
        // Remove el entities and el components
        for (entity, linking) in iter {
            // Remove the entities from the scene and from the archetype
            let index = linking.index;
            self.entities.swap_remove(index);

            // Update the linkings of the entity that was pushed in it's place (swap_remove)
            if index < (self.entities.len()) {
                let linkings = &mut entities[self.entities[index]];
                linkings.index = index;
            }

            entities.remove(entity);

            // Remove the components and decompose them
            for (mask, input) in self.table_mut() {
                input.states_mut().swap_remove(index);

                // Add the "removal" column in case it doesn't exist
                let output = removed
                    .entry(*mask)
                    .or_insert_with(|| input.components().clone_default());
                input
                    .components_mut()
                    .swap_remove_move(index, &mut **output);
            }
        }
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        log::debug!(
            "Reserving {} additional elements for archetype {}",
            additional,
            self.mask
        );
        self.entities.reserve(additional);

        // Reserve more memory for the columns
        for (_, column) in self.table.iter_mut() {
            column.reserve(additional);
        }
    }

    // Shrink the memory allocation used by this archetype
    pub fn shrink_to_fit(&mut self) {
        self.entities.shrink_to_fit();
        self.table.shrink_to_fit();

        // Shrink the actual column allocation
        for (_, column) in self.table.iter_mut() {
            column.shrink_to_fit();
        }
    }

    // Clear the archetype completely from entities
    pub(crate) fn clear(&mut self) {
        for (_, column) in self.table.iter_mut() {
            column.clear();
        }

        self.entities.clear();
    }

    // Get the number of entities that reference this archetype
    pub fn len(&self) -> usize {
        let len = self.entities.len();
        for (_, c) in &self.table {
            debug_assert_eq!(c.len(), len);
        }

        len
    }

    // Check if the archetype is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    // Get the entity slice immutably
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // Get the unique archetype mask
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Try to get an immutable reference to an untyped column of a specific component
    pub fn untyped_column<T: Component>(&self) -> Option<&UntypedColumn> {
        self.table.get(&mask::<T>())
    }

    // Try to get a mutable reference to an untyped column of a specific component
    pub(crate) fn untyped_column_mut<T: Component>(&mut self) -> Option<&mut UntypedColumn> {
        self.table.get_mut(&mask::<T>())
    }

    // Try to get an immutable reference to a typed column of a specific component
    pub fn column<T: Component>(&self) -> Option<(&Vec<T>, &StateColumn)> {
        self.untyped_column::<T>().map(|c| c.as_::<T>().unwrap())
    }

    // Try to get a mutable reference to a typed column of a specific component
    pub(crate) fn column_mut<T: Component>(&mut self) -> Option<(&mut Vec<T>, &mut StateColumn)> {
        self.untyped_column_mut::<T>()
            .map(|c| c.as_mut_::<T>().unwrap())
    }

    // Try to get an immutable reference to the data vector of a specific component
    pub fn components<T: Component>(&self) -> Option<&Vec<T>> {
        self.column::<T>().map(|(vec, _)| vec)
    }

    // Try to get a mutable reference to a data vector of a specific component
    pub(crate) fn components_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        self.column_mut::<T>().map(|(vec, _)| vec)
    }

    // Try to get an immutable reference to the states of a specific component
    pub fn states<T: Component>(&self) -> Option<&StateColumn> {
        self.untyped_column::<T>().map(|c| c.states())
    }

    // Try to get a mutable reference to the states of a specific component
    pub(crate) fn states_mut<T: Component>(&mut self) -> Option<&mut StateColumn> {
        self.untyped_column_mut::<T>().map(|c| c.states_mut())
    }

    // Get the internal table immutably
    pub fn table(&self) -> &Table {
        &self.table
    }

    // Get the internal table mutably
    pub(crate) fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

// This will get two different archetypes using their masks
// This assumes that the archetypes exist already in the set, and that we are using different masks
fn split(set: &mut ArchetypeSet, mask1: Mask, mask2: Mask) -> (&mut Archetype, &mut Archetype) {
    assert_ne!(mask1, mask2);
    let a1 = set.get_mut(&mask1).unwrap() as *mut Archetype;
    let a2 = set.get_mut(&mask2).unwrap() as *mut Archetype;
    unsafe {
        let a1 = &mut *a1;
        let a2 = &mut *a2;
        (a1, a2)
    }
}

// Initialize a new archetype for when we add a bundle to an entity
fn init_archetype_added_bundle<B: Bundle>(
    archetypes: &MaskHashMap<Archetype>,
    old: Mask,
    new: Mask,
) -> Archetype {
    let current = archetypes.get(&old).unwrap();
    let base_columns = current
        .table
        .iter()
        .map(|(mask, column)| (*mask, column.clone_default()));
    let columns = base_columns.chain(
        B::default_vectors()
            .into_iter()
            .map(|(mask, vec)| (mask, UntypedColumn::new(vec))),
    );

    Archetype {
        mask: new,
        table: Table::from_iter(columns),
        entities: Default::default(),
    }
}

// Initialize a new archetype for when we remove a bundle from a bundle
fn init_archetype_removed_bundle<B: Bundle>(
    archetypes: &MaskHashMap<Archetype>,
    old: Mask,
    new: Mask,
) -> Archetype {
    let current = archetypes.get(&old).unwrap();
    let columns = current
        .table
        .iter()
        .filter(|(mask, _)| new.contains(**mask))
        .filter(|(mask, _)| Mask::contains(&new, **mask))
        .map(|(mask, table)| (*mask, table.clone_default()));

    Archetype {
        mask: new,
        table: MaskHashMap::from_iter(columns),
        entities: Default::default(),
    }
}

// Add some new components onto an entity, forcing it to switch archetypes
pub(crate) fn add_bundle<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
    bundle: B,
) -> Option<()> {
    assert!(
        B::is_valid(),
        "Bundle is not valid, check the bundle for component collisions"
    );

    // Get the old and new masks
    let bundle_mask = B::reduce(|a, b| a | b);
    let old = entities[entity].mask;
    let new = entities[entity].mask | bundle_mask;

    // Nothing changed, don't execute
    if new == old {
        return None;
    }

    // Create the new target archetype if needed
    if !archetypes.contains_key(&new) {
        let arch = init_archetype_added_bundle::<B>(archetypes, old, new);
        archetypes.insert(new, arch);

        log::debug!("Created new archetype with mask {} (added bundle)", new);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.get(entity).unwrap();
    let index = linkings.index();

    // Move the components and states from one archetype to the other
    for (mask, input) in current.table.iter_mut() {
        let output = target.table.get_mut(mask).unwrap();
        input
            .components_mut()
            .swap_remove_move(index, output.components_mut());
        input
            .states_mut()
            .swap_remove_move(index, output.states_mut());
    }

    // Add the extra components to the archetype
    B::extend_from_iter(target, [bundle]).unwrap();

    for (mask, current) in current.table.iter() {
        log::debug!("Current Mask: {:?}, len: {}", mask, current.len());
    }

    for (mask, current) in target.table.iter() {
        log::debug!("Target Mask: {:?}, len: {}", mask, current.len());
    }

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target.entities.push(entity);
    linkings.index = target.len() - 1;
    linkings.mask = target.mask;

    Some(())
}

// Remove some old components from an entity, forcing it to switch archetypes
// This assumes that the OwnedBundle type is valid for this use case
pub(crate) fn remove_bundle<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
    removed: &mut RemovedComponents,
) -> bool {
    assert!(
        B::is_valid(),
        "Bundle is not valid, check the bundle for component collisions"
    );

    // Get the old and new masks
    let old = entities[entity].mask;
    let bundle_mask = B::reduce(|a, b| a | b);
    let new = entities[entity].mask & !bundle_mask;

    // Check if we even have the bundle stored
    if !old.contains(bundle_mask) {
        return false;
    }

    // Create the new target archetype if needed
    if !archetypes.contains_key(&new) {
        let arch = init_archetype_removed_bundle::<B>(archetypes, old, new);
        archetypes.insert(new, arch);

        log::debug!("Created new archetype with mask {} (removed bundle)", new);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = if let Some(entity) = entities.get(entity) {
        entity
    } else {
        return false;
    };
    let index = linkings.index();

    for (mask, current) in current.table.iter() {
        log::debug!("Removal Current Mask: {:?}, len: {}", mask, current.len());
    }

    for (mask, current) in target.table.iter() {
        log::debug!("Removal Target Mask: {:?}, len: {}", mask, current.len());
    }

    // Move the components and states from one archetype to the other (flipped)
    for (mask, output) in target.table.iter_mut() {
        let input = current.table.get_mut(mask).unwrap();
        input
            .components_mut()
            .swap_remove_move(index, output.components_mut());
        input
            .states_mut()
            .swap_remove_move(index, output.states_mut());
    }

    // Dissociate the bunle intop it's raw components
    for (mask, input) in current
        .table
        .iter_mut()
        .filter(|(mask, _)| bundle_mask.contains(**mask))
    {
        let entry = removed
            .entry(*mask)
            .or_insert_with(|| input.components().clone_default());
        let data = &mut **entry;
        input.components_mut().swap_remove_move(index, data);
        input.states_mut().swap_remove(index);
    }

    for (mask, current) in target.table.iter() {
        log::debug!("Removal Target Mask: {:?}, len: {}", mask, current.len());
    }

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target.entities.push(entity);
    linkings.index = target.len() - 1;
    linkings.mask = target.mask;

    true
}
