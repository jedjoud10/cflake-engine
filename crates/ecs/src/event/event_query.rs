use crate::{component::ComponentQuery, global::GlobalFetchKey};

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
    // Deconstruct the EventKey into all of it's fields
    pub fn decompose(self) -> Option<(ComponentQuery, GlobalFetchKey)> {
        Some((self.cquery?, GlobalFetchKey(())))
    }
}
