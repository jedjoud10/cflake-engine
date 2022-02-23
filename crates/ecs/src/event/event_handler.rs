use crate::event::EventKey;
// An event handler that stores all the system events
pub struct EventHandler<World> {
    evn_run_system: Vec<fn(&mut World, EventKey)>,
    evn_added_entity: Vec<fn(&mut World, EventKey)>,
    evn_removed_entity: Vec<fn(&mut World, EventKey)>,
}

impl<World> Default for EventHandler<World> {
    fn default() -> Self {
        Self {
            evn_run_system: Default::default(),
            evn_added_entity: Default::default(),
            evn_removed_entity: Default::default(),
        }
    }
}

impl<World> EventHandler<World> {
    // Add a "Run System" event to the EventHandler
    pub fn add_run_event(&mut self, event: fn(&mut World, EventKey)) -> usize {
        self.evn_run_system.push(event);
        self.evn_run_system.len() - 1
    }
    // Add a "Added Entity" event to the EventHandler
    pub fn add_added_entity_event(&mut self, event: fn(&mut World, EventKey)) -> usize {
        self.evn_added_entity.push(event);
        self.evn_added_entity.len() - 1
    }
    // Add a "Removed Entity" event to the EventHandler
    pub fn add_removed_entity_event(&mut self, event: fn(&mut World, EventKey)) -> usize {
        self.evn_removed_entity.push(event);
        self.evn_removed_entity.len() - 1
    }
    // Get the "Run System" event for a specific index
    pub fn get_run_event(&self, idx: Option<usize>) -> Option<&fn(&mut World, EventKey)> {
        if let Some(idx) = idx {
            self.evn_run_system.get(idx)
        } else {
            None
        }
    }
    // Get the "Added Entity" event for a specific index
    pub fn get_added_entity_event(&self, idx: Option<usize>) -> Option<&fn(&mut World, EventKey)> {
        if let Some(idx) = idx {
            self.evn_added_entity.get(idx)
        } else {
            None
        }
    }
    // Get the "Removed Entity" event for a specific index
    pub fn get_removed_entity_event(
        &self,
        idx: Option<usize>,
    ) -> Option<&fn(&mut World, EventKey)> {
        if let Some(idx) = idx {
            self.evn_removed_entity.get(idx)
        } else {
            None
        }
    }
}
