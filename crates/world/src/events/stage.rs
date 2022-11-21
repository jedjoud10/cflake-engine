use std::any::TypeId;

use crate::Caller;

// Names are shared around since we clone them frequently
pub type StageId = (&'static str, TypeId);

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
            Rule::Before(p) => *p,
            Rule::After(p) => *p,
        }
    }
}

// Get the StageId of given linked stage event function cause type_name_of_val is unstable
// TODO: Watch quintuplets movie
pub(crate) fn id<E: 'static>(event: E) -> (StageId, E) {
    let id = TypeId::of::<E>();
    let name = std::any::type_name::<E>();
    ((name, id), event)
}

// Default functions
pub fn user() {}
pub fn post_user() {}

// Create the default rules for a default node
pub(super) fn default_rules<C: Caller>() -> Vec<Rule> {
    let post = id(post_user).0;
    let user = id(user).0;
    vec![Rule::Before(post), Rule::After(user)]
}
