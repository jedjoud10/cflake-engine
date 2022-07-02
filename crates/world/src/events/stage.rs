use std::rc::Rc;
use ahash::AHashMap;

use crate::StageError;

// Names are shared around since we clone them frequently
pub type StageKey = Rc<str>;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Clone)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(StageKey),

    // This hints that the stage should be executed after other
    After(StageKey),
}

impl Rule {
    // Get the current parent of the current strict node
    pub(super) fn parent(&self) -> StageKey {
        match self {
            Rule::Before(p) => p.clone(),
            Rule::After(p) => p.clone(),
        }
    }
}

// This is a list of reserved names for internal stages that we cannot reproduce
const RESERVED_NAMES: &[&str] = &[super::pipeline::EXEC_STAGE_NAME];

// Stages are are a way for us to sort and prioritize certain events before others
// Stages will be converted to Nodes whenever they get inserted into a pipeline
#[derive(Clone)]
pub struct Stage {
    // The user defined name of the current stage
    name: StageKey,

    // The direction of this rule compared to other rules
    rules: Vec<Rule>,
}

impl Stage {
    // Create a new empty stage with no rules (invalid)
    pub fn new(name: impl Into<StageKey>) -> Self {
        Self { name: name.into(), rules: Vec::new()  }
    }

    // Get a copy of the underlying stage name
    pub fn name(&self) -> StageKey {
        self.name.clone()
    }

    // Add a "before" rule to the current stage
    pub fn set_before(mut self, other: impl Into<StageKey>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a "after" rule to the current stage
    pub fn set_after(mut self, other: impl Into<StageKey>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self
    }

    // Check if the stage is valid, and if it isn't, return the proper error
    pub(super) fn validate(self) -> Result<Self, StageError> {
        if self.rules.is_empty() {
            return Err(StageError::MissingRules);
        } else if self.name.is_empty() || RESERVED_NAMES.contains(&self.name.as_ref()) {
            return Err(StageError::InvalidName);
        }

        Ok(self)
    }

    // Convert the stage into a set of rules
    pub(super) fn into_rules(self) -> Vec<Rule> {
        self.rules
    }
}