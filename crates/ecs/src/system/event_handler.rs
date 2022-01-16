use crate::component::ComponentQuery;

// An event handler that stores all the system events
pub struct EventHandler<Context> {
    // Run events
    run_systems: Vec<fn(Context, ComponentQuery)>,
}

impl<Context> Default for EventHandler<Context> {
    fn default() -> Self {
        Self { run_systems: Default::default() }
    }
}

impl<Context> EventHandler<Context> {
    // Add an event to the EventHandler
    pub fn add_run_event(&mut self, event: fn(Context, ComponentQuery)) -> usize {
        self.run_systems.push(event);
        self.run_systems.len() - 1
    }
    // Get the run event for a specific index
    pub fn get_run_event(&self, idx: isize) -> Option<&fn(Context, ComponentQuery)> {
        if idx == -1 {
            return None;
        }
        self.run_systems.get(idx as usize)
    }
}
