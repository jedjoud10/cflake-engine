use crate::event::EventKey;

use super::Event;

// Some data that is created and returned whenever we want to execute a system
pub struct SystemExecutionData<World> {
    // Some events and their queries
    pub(crate) run: (Event<World>, EventKey),
    pub(crate) run_fixed: (Event<World>, EventKey),
    pub(crate) added_entity: (Event<World>, EventKey),
    pub(crate) removed_entity: (Event<World>, EventKey),
}

impl<World> SystemExecutionData<World> {
    // Actually execute the system update
    pub fn run(self, world: &mut World) {
        // Run the "Added Entity" and "Removed Entity" events first, then we can run the "Run System" event
        if let Some(evn_added_entity) = self.added_entity.0 {
            evn_added_entity(world, self.added_entity.1);
        }
        if let Some(evn_removed_entity) = self.removed_entity.0 {
            evn_removed_entity(world, self.removed_entity.1);
        }
        if let Some(run_system_evn) = self.run.0 {
            run_system_evn(world, self.run.1);
        }
        // Run the fixed time event
    }
}
