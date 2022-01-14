use std::{cell::RefCell, marker::PhantomData, ffi::c_void};

use crate::{linked_components::{LinkedComponents, ComponentQuery}, Component, ComponentID, EnclosedComponent, Entity, EntityID, ECSManager};
use ahash::{AHashMap, AHashSet};
use bitfield::Bitfield;
use worker_threads::ThreadPool;

// A system that updates specific components in parallel
pub struct System<RefContext: 'static, MutContext: 'static> {
    // Our Component Bitfield
    cbitfield: Bitfield<u32>, 
    // Events
    run_system: fn(context: &mut MutContext, components: ComponentQuery),
    added_component_group: fn(context: &mut MutContext, components: ComponentQuery),
    removed_component_group: fn(context: &mut MutContext, component: ComponentQuery),
    phantom_: PhantomData<*const RefContext>,
    // The stored linked components
    linked_components: AHashMap<EntityID, LinkedComponents>,
}

// Initialization of the system
impl<RefContext: 'static, MutContext: 'static> System<RefContext, MutContext> {
    // Create a new system
    pub fn new() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            run_system: |context, query| {},
            added_component_group: |context, query| {},
            removed_component_group: |context, query| {},
            phantom_: PhantomData::default(),
            linked_components: AHashMap::new(),
        }
    }
}

// System code
impl<RefContext: 'static, MutContext: 'static> System<RefContext, MutContext> {
    // Add a component to this system's component bitfield id
    pub fn link<U: Component>(&mut self) {
        let c = crate::registry::get_component_bitfield::<U>();
        self.cbitfield = self.cbitfield.add(&c);
    }
    // Set the run system event
    pub fn set_event(&mut self, run_system: fn(&mut MutContext, ComponentQuery)) {
        self.run_system = run_system;
    }
    // Check if we can add an entity (It's cbitfield became adequate for our system or the entity was added from the world)
    // If we can add it, then just do that
    pub(crate) fn check_add_entity(&mut self, ecs_manager: &ECSManager<RefContext, MutContext>, cbitfield: Bitfield<u32>, id: EntityID) {
        if cbitfield.contains(&self.cbitfield) && !self.cbitfield.empty() {
            let entity = ecs_manager.entity(&id).unwrap();
            let linked_components = LinkedComponents::new(&id, entity, &ecs_manager.components, &self.cbitfield);
            self.linked_components.insert(id, linked_components);
        }
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&mut self, id: EntityID) {
        self.linked_components.remove(&id);
    }
    // Run the system for a single iteration
    pub fn run_system(&self, mut_context: &mut MutContext, ecs_manager: &ECSManager<RefContext, MutContext>) {
        // These components are filtered for us
        let components = &ecs_manager.components;    
        let i = std::time::Instant::now();
        dbg!(i.elapsed());
        // Create the component query
        let query = ComponentQuery {
            linked_components: &self.linked_components,
            pool: { (&ecs_manager.pool as *const ThreadPool<RefContext, LinkedComponents>) as *const c_void },
        };
        let run_system_evn = self.run_system;
        // Run the "run system" event
        run_system_evn(mut_context, query);
    }
}
