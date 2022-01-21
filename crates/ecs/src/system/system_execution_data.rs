use crate::component::ComponentQuery;

// Some data that is created and returned whenever we want to execute a system
pub struct SystemExecutionData<Context> {
    // The execution event
    pub(crate) run_event: Option<fn(Context, ComponentQuery)>,
    // Query
    pub(crate) query: ComponentQuery,
}

impl<Context> SystemExecutionData<Context> {
    // Actually execute the system update
    pub fn run(mut self, context: Context) {
        if let Some(run_system_evn) = self.run_event {
            // Run the "run system" event
            run_system_evn(context, self.query);
        }
    }
}
