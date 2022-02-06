use crate::{component::ComponentQuery};

// Some data that will be passed to each of the systems' events whenever we execute them
// This can be used as a "key" to access global components, since it will take reference to this EventQuery instead of the manager
pub struct EventKey {
    // The optional component query for this specific event
    cquery: Option<ComponentQuery>,
}

impl EventKey {
    // Create a new event key using some component query data
    pub(crate) fn new(cquery: ComponentQuery) -> Self {
        Self { cquery: Some(cquery) }
    }
    // Try to get the component query
    pub fn get_query(self) -> Option<ComponentQuery> {
        self.cquery
    }
}
