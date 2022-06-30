use std::ops::{Bound, RangeBounds, RangeFull};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Debug, Clone, Copy)]
enum Rule {
    Before(Key),
    After(Key)
    Between(Key, Key),
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Rules that apply to this stage
    rules: Vec<Rule>,
}

impl Stage {
    // Create a stage that will execute anytime, since it has no rules
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rules: Default::default()}
    }

    // Add a rule to this stage hinting it that it should execute before other
    pub fn before(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a rule to this stage hinting it that it should execute after other
    pub fn after(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self
    }

    // Free nodes are nodes that don't have any rules associated to them
    pub fn free(&self) -> bool {
        self.rules.is_empty()
    }
}


// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // Check if we have any duplicate stages
    let dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    let vec = dedupped.into_iter().map(|(_, stage)| stage).collect::<Vec<_>>();

    Vec::new()
}


#[test]
fn test() {
    let node1 = Stage::new("main").after("mycock");
    let node2 = Stage::new("substage").after("main");
    let node3 = Stage::new("sub-substage").before("main");
    evaluate(vec![node1, node2, node3]);
}