use core::fmt;

use crate::entity::EntityKey;

// An error related to the entities
#[derive(Debug)]
pub struct EntityError {
    details: String,
    key: EntityKey,
}

impl EntityError {
    pub(crate) fn new(msg: String, key: EntityKey) -> Self {
        Self { details: msg, key }
    }
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}. EntityKey: {:?}", self.details, self.key)
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
    pub(crate) details: String,
}

impl ComponentError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ComponentError {
    fn description(&self) -> &str {
        &self.details
    }
}
// An error that might occur when trying to link component
#[derive(Debug)]
pub struct ComponentLinkingError {
    pub(crate) details: String,
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
// An error that might occur when trying to unlink component
#[derive(Debug)]
pub struct ComponentUnlinkError {
    details: String,
}

impl ComponentUnlinkError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
}

impl fmt::Display for ComponentUnlinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ComponentUnlinkError {
    fn description(&self) -> &str {
        &self.details
    }
}
// Error that occurs whenever we try to build a system during a frame
#[derive(Debug)]
pub struct SystemBuildingError;

impl fmt::Display for SystemBuildingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot build system during ECS frame!")
    }
}

impl std::error::Error for SystemBuildingError {
    fn description(&self) -> &str {
        "Cannot build system during ECS frame!"
    }
}
