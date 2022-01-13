use std::cell::RefCell;

use crate::{linked_components::LinkedComponents, Component, ComponentID, EnclosedComponent, Entity, EntityID, ECSManager};
use ahash::{AHashMap, AHashSet};
use bitfield::Bitfield;

// A system that updates specific components in parallel
pub struct System<C: 'static> {
    // Our Component Bitfield
    cbitfield: Bitfield<u32>, 
    // Event
    update_components_event: fn(context: &C, components: &mut LinkedComponents),
    // The entity IDs
    entities: AHashSet<EntityID>,
    // Are we updating the components in parallel?
    multithreading: bool,
}

// Initialization of the system
impl<C: 'static> System<C> {
    // Create a new system
    pub fn new() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            update_components_event: |_, _| {},
            entities: AHashSet::default(),
            multithreading: false,
        }
    }
}

// System code
impl<C: 'static> System<C> {
    // Add a component to this system's component bitfield id
    pub fn link<U: Component>(&mut self) {
        let c = crate::registry::get_component_bitfield::<U>();
        self.cbitfield = self.cbitfield.add(&c);
    }
    // Enable multithreading, so whenever we update the components, we will update them in parallel
    pub fn enable_multithreading(&mut self) {
        self.multithreading = true;
    } 
    // Set the update components event
    pub fn set_event(&mut self, event: fn(context: &C, components: &mut LinkedComponents)) {
        self.update_components_event = event;
    }
    // Check if we can add an entity (It's cbitfield became adequate for our system or the entity was added from the world)
    // If we can add it, then just do that
    pub(crate) fn check_add_entity(&mut self, cbitfield: Bitfield<u32>, id: EntityID) {
        if cbitfield.contains(&self.cbitfield) && !self.cbitfield.empty() {
            self.entities.insert(id);
        }
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&mut self, id: EntityID) {
        self.entities.remove(&id);
    }
    // Run the system for a single iteration
    // This will use the components data given by the world to run all the component updates in PARALLEL
    // The components get mutated in parallel, though the system is NOT stored on another thread
    pub fn run_system(&self, context: &C, ecs_manager: &ECSManager<C>) {
        // These components are filtered for us
        let components = &ecs_manager.components;        
        let evn = self.update_components_event;
        let components = self.entities.iter().map(|id| LinkedComponents::new(id, components, &self.cbitfield));
        if self.multithreading {
            // Multi threadingggg
            let mut components = components.collect::<Vec<LinkedComponents>>();
            ecs_manager.pool.execute(&mut components, context, evn)
        } else {
            // Not multithreaded, just the single threaded manner
            for mut linked_components in components {
                evn(context, &mut linked_components);
            }
        }
    }
}
