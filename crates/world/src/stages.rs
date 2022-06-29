use std::ops::{Bound, RangeBounds, RangeFull};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
enum Rule {
    Before(Key),
    After(Key)
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Rules that apply to this stage
    rules: Vec<Rule>,

    // Current stage group. Becomes None if we have any rules
    level: Option<usize>,
}

impl Stage {
    // Create a stage that will execute anytime, since it has no rules
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rules: Default::default(), level: Some(0) }
    }

    // Add a rule to this stage hinting it that it should execute before other
    pub fn before(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self.level = None;
        self
    }

    // Add a rule to this stage hinting it that it should execute after other
    pub fn after(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self.level = None;
        self
    }

    // Free nodes are nodes that don't have any rules associated to them
    pub fn free(&self) -> bool {
        self.rules.is_empty()
    }
}


// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // Make sure there are no duplicates, and index by name 
    let map: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|stage| (stage.name, stage)));

    // This is the list of nodes that we must evaluate
    let mut eval: Vec<Key> = Vec::new();

    // Evaluate the nodes that don't have any rules first
    for node in map.values().filter(|s| Stage::free(s)) {
        eval.push(node.name);
    }

    // A multidimensional registry to contain all the levels and nodes
    let mut levels: Vec<Vec<Key>> = Vec::new();

    // Push a new key into the levels
    fn insert_key(key: Key, idx: usize, levels: &mut Vec<Vec<Key>>) {
        if levels.len() < (idx + 1) {
            levels.push(Vec::new());
        } else {
            let level = &mut levels[idx];
            level.push(key);
        }
    }

    // Recursively evaluate the nodes
    loop {
        for name in eval.drain(..) {
            // Try to insert the node into the proper levels
            let node = &map[name];

            // Check if the node has rules
            if node.free() {
                insert_key(node.name, 0, &mut levels);

                // We must now evaluate the nodes that use the current node, aka the children
            } else {
                //insert_key(node.name, )

                // We have multiple rules, so we must restrict our node
                for rule in node.rules.iter() {
                    match rule {
                        Rule::Before(other) => {
                            // Check if the current node position is on the same level as other
                            //    if it is, then make a new level before the leve of other and swap
                            // Move the node before other                            
                        },
                        Rule::After(other) => {

                        },
                    }
                }
            }

        }
    }

    Vec::new()
}
