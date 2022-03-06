use std::{cell::{Ref, RefCell}, rc::Rc};

use crate::{entity::EntitySet, component::ComponentSet, system::{SystemSet, SystemSettings, Systems, System}};

// The Entity Component System manager that will handle everything ECS related
pub struct ECSManager<World> {
    pub entities: EntitySet<World>,
    pub components: ComponentSet<World>,
    pub systems: SystemSet<World>,
}

impl<World> Default for ECSManager<World> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            components: Default::default(),
            systems: Default::default(),
        }
    }
}

// Global code for the Entities, Components, and Systems
impl<World> ECSManager<World> {  
    // Create the proper execution settings for systems, and return them
    pub fn ready(&self) -> (Rc<RefCell<Vec<System<World>>>>, SystemSettings) {
        (self.systems.inner.clone(), SystemSettings {
            to_remove: self.components.to_remove.clone(),
        })
    }  
    // Execute a bunch of systems
    pub fn execute_systems(systems: Ref<Vec<System<World>>>, world: &mut World, settings: SystemSettings) {
        for system in systems.iter() {
            system.run_system(world, settings.clone());
        }
    } 
    // Run the systems in sync, but their component updates are not
    // Used only for testing
    #[allow(dead_code)]
    pub(crate) fn run_systems(&self, world: &mut World) {
        let (systems, settings) = self.ready();
        Self::execute_systems(systems.borrow(), world, settings);
    }
    /* #endregion */
    // Finish update of the ECS manager
    pub fn finish_update(&mut self) {
        self.components.clear().unwrap();
    }
}
