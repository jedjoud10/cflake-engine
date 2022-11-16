use std::{marker::PhantomData, any::TypeId, mem::MaybeUninit, rc::Rc};

use crate::{StageError, RESERVED, Caller, Event};

// Names are shared around since we clone them frequently
pub type StageId = (TypeId, &'static str);

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Clone, Debug)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(StageId),

    // This hints that the stage should be executed after other
    After(StageId),
}

impl Rule {
    // Get the current parent of the current strict node
    pub(super) fn parent(&self) -> StageId {
        match self {
            Rule::Before(p) => p.clone(),
            Rule::After(p) => p.clone(),
        }
    }
}

// Stages are are a way for us to sort and prioritize certain events before others
// This is a typed stage that contains the event type (caller)
#[derive(Clone)]
pub struct Stage<C: Caller> {
    // The user defined name of the current stage
    pub(super) name: Option<StageId>,

    // The direction of this rule compared to other rules
    pub(super) rules: Vec<Rule>,

    // Type of the event that will use this stage
    _phantom: PhantomData<C>,
}


// Get the StageId of given linked stage event function cause type_name_of_val is unstable
pub(crate) fn id<E: 'static>(event: E) -> (StageId, E) {
    let id = (TypeId::of::<E>(), std::any::type_name::<E>().into());
    (id, event)
}

// Default functions
pub fn user() {}
pub fn post_user() {}

// Create the default rules for a default node
pub(super) fn default_rules() -> Vec<Rule> {
    let post = id(post_user).0;
    let user = id(user).0;
    vec![Rule::Before(post), Rule::After(user)]
}

impl<C: Caller> Stage<C> {
    // Create a new empty stage with no rules (invalid)
    pub fn new() -> Self {
        Self {
            name: None,
            rules: Vec::new(),
            _phantom: PhantomData,
        }
    }

    // Get a copy of the underlying stage key
    // This assumes that the stage key was already set internally
    pub fn name(&self) -> StageId {
        self.name.unwrap()
    }

    // Add a "before" rule to the current stage
    pub fn before<ID>(mut self, other: impl Event<C, ID> + 'static) -> Self {
        let (id, _) = id(other);
        self.rules.push(Rule::Before(id));
        self
    }

    // Add a "after" rule to the current stage
    pub fn after<ID>(mut self, other: impl Event<C, ID> + 'static) -> Self {
        let (id, _) = id(other);
        self.rules.push(Rule::After(id));
        self
    }

    // Convert the stage into a set of rules
    pub(super) fn into_rules(self) -> Vec<Rule> {
        self.rules
    }
}
