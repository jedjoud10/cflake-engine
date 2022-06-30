use std::{ops::{Bound, RangeBounds, RangeFull, Range}, collections::HashMap, rc::Rc};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Debug, Clone, Copy)]
enum Rule {
    // This hints that the stage should be executed before other
    Before(Key),

    // This hints that the stage should be executed after other
    After(Key),
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Rules that restrict this stage
    rules: Vec<Rule>,
}

impl Stage {
    // Create a stage with no rules. It is a free stage
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rules: Vec::default() }
    }

    // Add a "before" rule to the current stage
    pub fn before(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a "after" rule to the current stage
    pub fn after(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self
    }
}
// Number of maximum calculations allowed before we detect a cyclic references
const CYCLIC_REFERENCE_THRESHOLD: usize = 8;

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// This will sort the stages based on their rules 
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    enum BoxedRule {
        Before(Rc<Node>),
        After(Rc<Node>),
    }
    
    struct Node {
        name: Key,
        rules: Vec<BoxedRule>,
    }
}

#[test]
fn test() {
    let other = Stage::new("test").before("input");
    let node1 = Stage::new("main");
    let input = Stage::new("input").before("main");
    let rendering = Stage::new("renering").after("input").after("main");
    let sorted = evaluate(vec![other, node1, input, rendering]);

    for stage in sorted {
        dbg!(stage.name);
    }
}