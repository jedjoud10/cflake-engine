use std::{cell::RefCell, marker::PhantomData, ffi::c_void, sync::{RwLock, Arc}};

use crate::{linked_components::{LinkedComponents, ComponentQuery}, Component, ComponentID, EnclosedComponent, Entity, EntityID, ECSManager};
use ahash::{AHashMap, AHashSet};
use bitfield::Bitfield;

// A system that updates specific components in parallel
pub struct System {
    // Our Component Bitfield
    cbitfield: Bitfield<u32>, 
    // Events
    run_system: Option<Box<dyn Fn(ComponentQuery)>>,
    added_component_group: Option<Box<dyn Fn(ComponentQuery)>>,
    removed_component_group: Option<Box<dyn Fn(ComponentQuery)>>,
    entities: AHashSet<EntityID>,
}

// Initialization of the system
impl System {
    // Create a new system
    pub fn new() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            run_system: None,
            added_component_group: None,
            removed_component_group: None,
            entities: AHashSet::new(),
        }
    }
}

// System code
impl System {
    // Add a component to this system's component bitfield id
    pub fn link<U: Component>(&mut self) {
        let c = crate::registry::get_component_bitfield::<U>();
        self.cbitfield = self.cbitfield.add(&c);
    }
    // Set the a ref context system event
    pub fn set_event<'a, 'b: 'a, RefContext: Clone + 'static>(&'a mut self, refc: RefContext, run_system: &'a fn(RefContext, ComponentQuery)) {
        let event = |query: ComponentQuery| {
            run_system(refc.clone(), query);
        };
        self.run_system = Some(Box::new(event));
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
    pub fn run_system(&self, ecs_manager: &ECSManager) {
        // These components are filtered for us
        let components = &ecs_manager.components;    
        let i = std::time::Instant::now();
        // Create the component query
        let mut components = self.entities.iter().map(|id| {
            let entity = ecs_manager.entity(id).unwrap();            
            let linked_components = LinkedComponents::new(id, entity, components, &entity.cbitfield); 
            linked_components
        }).collect::<Vec<_>>();
        
        let query = ComponentQuery {
            linked_components: components,
            thread_pool: &ecs_manager.thread_pool,
        };
        if let Some(run_system_evn) = self.run_system.as_ref() {
            // Run the "run system" event
            run_system_evn(query);
        }
        //dbg!(i.elapsed());
    }
}
