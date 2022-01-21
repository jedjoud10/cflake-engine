use crate::component::ComponentQuery;

// Some data that is created and returned whenever we want to execute a system
pub struct SystemExecutionData<Context> {
    // Some events
    pub(crate) evn_run: Option<fn(Context, ComponentQuery)>,
    pub(crate) evn_added_entity: Option<fn(Context, ComponentQuery)>,
    pub(crate) evn_removed_entity: Option<fn(Context, ComponentQuery)>,
    // Queries
    pub(crate) evn_run_query: ComponentQuery,
    pub(crate) evn_added_entity_query: ComponentQuery,
    pub(crate) evn_removed_entity_query: ComponentQuery,
}

impl<Context> SystemExecutionData<Context> {
    // Actually execute the system update
    pub fn run(mut self, context: Context) {
        if let Some(run_system_evn) = self.evn_run {
            // Run the "run system" event
            run_system_evn(context, self.evn_run_query);
        }
    }
}
