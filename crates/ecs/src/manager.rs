use getset::Getters;
use slotmap::SlotMap;

use crate::{entity::Entity, filtered, query, Archetype, EntityLinkings, Entry, Evaluate, LinkModifier, Mask, MaskMap, QueryLayout, StorageVec};

// Type aliases
pub type EntitySet = SlotMap<Entity, EntityLinkings>;
pub type ArchetypeSet = MaskMap<Archetype>;
pub(crate) type UniqueStoragesSet = MaskMap<Box<dyn StorageVec>>;

#[derive(Getters)]
pub struct EcsManager {
    #[getset(get = "pub")]
    pub(crate) entities: EntitySet,
    #[getset(get = "pub")]
    pub(crate) archetypes: ArchetypeSet,
    pub(crate) uniques: UniqueStoragesSet,

    // Others
    count: u64,
}

impl Default for EcsManager {
    fn default() -> Self {
        // Create the default empty archetype
        let uniques: UniqueStoragesSet = Default::default();
        let empty = Archetype::new(Mask::zero(), &uniques);

        Self {
            entities: Default::default(),
            archetypes: MaskMap::from_iter(std::iter::once((Mask::zero(), empty))),
            uniques,
            count: Default::default(),
        }
    }
}

impl EcsManager {
    // Prepare the Ecs Manager for one execution
    pub fn prepare(&mut self) {
        // Reset the archetype component mutation bits
        for (_, archetype) in self.archetypes.iter_mut() {
            archetype.prepare(self.count)
        }

        // Iteration counter that keeps track how many times we've run the ECS system
        self.count += 1;
    }

    // Modify an entity's component layout
    pub fn modify(&mut self, entity: Entity, function: impl FnOnce(Entity, &mut LinkModifier)) -> Option<()> {
        // Keep a copy of the linkings before we do anything
        let mut copied = *self.entities.get(entity)?;

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Apply the changes
        linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;
        Some(())
    }

    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<Entry> {
        Entry::new(self, entity)
    }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert(&mut self, function: impl FnOnce(Entity, &mut LinkModifier)) -> Entity {
        // Add le entity
        let entity = self.entities.insert(EntityLinkings::default());

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Since we are inserting this entity, the linkings are always default
        let mut linkings = EntityLinkings::default();
        linker.apply(&mut linkings);
        *self.entities.get_mut(entity).unwrap() = linkings;

        entity
    }

    // Remove an entity from the world
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Remove the entity from it's current archetype first
        Archetype::remove(&mut self.archetypes, &mut self.entities, entity, Mask::zero());

        // Then remove it from the manager
        self.entities.remove(entity).unwrap();
        Some(())
    }

    // Normal query without filter
    pub fn try_query<'a, Layout: QueryLayout<'a> + 'a>(&'a mut self) -> Option<impl Iterator<Item = Layout> + 'a> {
        Layout::validate().then(|| query(&self.archetypes))
    }
    // Create a query with a specific filter
    pub fn try_query_with<'a, Layout: QueryLayout<'a> + 'a, Filter: Evaluate>(&'a mut self, filter: Filter) -> Option<impl Iterator<Item = Layout> + 'a> {
        Layout::validate().then(|| filtered(&self.archetypes, filter))
    }
    // A view query that can only READ data, and never write to it
    // This will return None when it is unable to get a view query
    // TODO: Make use of Rust's type system to check for immutable borrows instead
    pub fn try_view<'a, Layout: QueryLayout<'a> + 'a>(&'a self) -> Option<impl Iterator<Item = Layout> + 'a> {
        let valid = Layout::combined().writing().empty() && Layout::validate();
        valid.then(|| query(&self.archetypes))
    }
    // View query with a specific filter
    pub fn try_view_with<'a, Layout: QueryLayout<'a> + 'a, Filter: Evaluate>(&'a self, filter: Filter) -> Option<impl Iterator<Item = Layout> + 'a> {
        let valid = Layout::combined().writing().empty() && Layout::validate();
        valid.then(|| filtered(&self.archetypes, filter))
    }
}
