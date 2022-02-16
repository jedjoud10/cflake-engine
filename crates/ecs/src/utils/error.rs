use crate::component::ComponentID;
use crate::entity::EntityID;
use core::fmt;

// An error related to the entities
#[derive(Debug)]
pub struct EntityError {
    details: String,
    id: EntityID,
}

impl EntityError {
    pub(crate) fn new(msg: String, id: EntityID) -> Self {
        Self { details: msg, id }
    }
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}. EntityID: {:?}", self.details, self.id)
    }
}

impl std::error::Error for EntityError {
    fn description(&self) -> &str {
        &self.details
    }
}

// An error related to the components
#[derive(Debug)]
pub struct ComponentError {
    details: String,
    id: Option<ComponentID>,
}

impl ComponentError {
    pub fn new(msg: String, id: ComponentID) -> Self {
        Self { details: msg, id: Some(id) }
    }
    pub const fn new_without_id(msg: String) -> Self {
        Self { details: msg, id: None }
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}. ComponentID: {:?}", self.details, self.id)
    }
}

impl std::error::Error for ComponentError {
    fn description(&self) -> &str {
        &self.details
    }
}
// An error related to the linkage of the components
#[derive(Debug)]
pub struct ComponentLinkingError {
    details: String,
}

impl ComponentLinkingError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
}

impl fmt::Display for ComponentLinkingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ComponentLinkingError {
    fn description(&self) -> &str {
        &self.details
    }
}
