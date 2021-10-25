use crate::{Entity, LoadState};

// Some load/unload struct that contains load/unload events for entities
pub struct LoadUnload {
    // When this entity changes it's load state
    pub entity_u_load_state: Option<(fn(&Entity), LoadState)>,    
}