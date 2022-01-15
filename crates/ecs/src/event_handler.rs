use crate::linked_components::ComponentQuery;

// An event handler that stores all the system events
pub struct EventHandler<RefContext> {
    // Run events
    run_systems: Vec<fn(RefContext, ComponentQuery)>,
}

impl<RefContext> EventHandler<RefContext> {
    // New
    pub fn new() -> Self {
        Self {
            run_systems: Vec::new(),
        }
    }
    // Add an event to the EventHandler
    pub fn add_run_event(&mut self, event: fn(RefContext, ComponentQuery)) -> usize {
        self.run_systems.push(event);
        self.run_systems.len() - 1
    }
    // Get the run event for a specific index
    pub fn get_run_event(&self, idx: isize) -> Option<&fn(RefContext, ComponentQuery)> {
        if idx == -1 { return None; }
        self.run_systems.get(idx as usize)
    }
}