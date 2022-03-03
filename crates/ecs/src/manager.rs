use crate::{entity::EntitySet, component::ComponentSet, system::SystemSet};

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
    // Run the systems in sync, but their component updates are not
    // Used only for testing
    #[allow(dead_code)]
    pub(crate) fn run_systems(&self, world: &mut World) {
        for system in self.systems.inner() {
            let execution_data = system.run_system(self);
            execution_data.run(world);
            system.clear();
        }
    }
    /* #endregion */
    // Finish update of the ECS manager
    pub fn finish_update(&mut self) {
        self.components.clear().unwrap();
    }
}
