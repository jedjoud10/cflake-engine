use crate::event::EventKey;

// Some data that is created and returned whenever we want to execute a system
pub struct SystemExecutionData<Context> {
    // Some events
    pub(crate) evn_run: Option<fn(&mut Context, EventKey)>,
    pub(crate) evn_added_entity: Option<fn(&mut Context, EventKey)>,
    pub(crate) evn_removed_entity: Option<fn(&mut Context, EventKey)>,
    // Queries
    pub(crate) evn_run_ekey: EventKey,
    pub(crate) evn_added_entity_ekey: EventKey,
    pub(crate) evn_removed_entity_ekey: EventKey,
}

impl<Context> SystemExecutionData<Context> {
    // Actually execute the system update
    pub fn run(self, context: &mut Context) {
        // Run the "Added Entity" and "Removed Entity" events first, then we can run the "Run System" event
        if let Some(evn_added_entity) = self.evn_added_entity {
            evn_added_entity(context, self.evn_added_entity_ekey);
        }
        if let Some(evn_removed_entity) = self.evn_removed_entity {
            evn_removed_entity(context, self.evn_removed_entity_ekey);
        }

        if let Some(run_system_evn) = self.evn_run {
            run_system_evn(context, self.evn_run_ekey);
        }
    }
}
