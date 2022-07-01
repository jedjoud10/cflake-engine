use std::ops::{Bound, RangeBounds, RangeFull};

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

    // The direction of this rule compared to other rules
    rules: Vec<Rule>,
}

impl Stage {
    // Create a stage that has no rules associated with it
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rules: Vec::new() }
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

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// The name of the main execution start stage
const EXEC_STAGE_NAME: &str = "exec";

// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // Convert the vector into a hashmap (this removes any duplicates)
    let mut dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    let keys: Vec<Key> = dedupped.keys().cloned().collect();

    // We must panic if we find any stages that have no rules
    for stage in dedupped.values() {
        if stage.rules.is_empty() {
            panic!()
        }
    }

    let mut indices = AHashMap::<Key, usize>::default();
    let mut vec = Vec::<Stage>::default();

    vec.push(Stage { name: EXEC_STAGE_NAME, rules: Vec::default() });
    indices.insert(EXEC_STAGE_NAME, 0);

    // This function will add a current stage into the main vector and sort it according to it's rules
    fn calc(key: Key, indices: &mut AHashMap<Key, usize>, dedupped: &mut AHashMap<Key, Stage>, vec: &mut Vec<Stage>) -> usize {
        if dedupped.contains_key(key) {
            // We must insert the stage into the main vector
            let stage = dedupped.remove(key).unwrap();
            let rules = stage.rules.as_slice();

            // Restrict the index of the stage based on it's rules
            let mut changed = true;
            let mut location = 0;

            while changed {
                changed = false;

                for rule in rules {
                    match rule {
                        Rule::Before(p) => {
                            if *p == EXEC_STAGE_NAME {
                                panic!()
                            }

                            let pl = calc(p, indices, dedupped, vec);

                            if location > pl {
                                location = pl - 1;
                                changed = true;
                            }
                        },
                        Rule::After(p) => {
                            let pl = calc(p, indices, dedupped, vec);

                            if location <= pl {
                                location = pl + 1;
                                changed = true;
                            }
                        },
                    }
                }
            }

            indices.insert(stage.name, vec.len());
            println!("key {key}, loc: {location}");
            vec.insert(location, stage);

            for (i, stage) in vec.iter_mut().enumerate() {
                *indices.get_mut(stage.name).unwrap() = i;
            }

            location
        } else {
            // Fetch the cached location instead
            indices[key]
        }
    }

    // Add the stages into a vector and start sorting them
    for key in keys {
        calc(key, &mut indices, &mut dedupped, &mut vec);
    }

    vec
}


#[test]
fn test() {
    let input = Stage::new("input").after("exec");
    let rendering = Stage::new("rendering").after("input");
    //let post_input = Stage::new("post input").after("input").after("rendering");
    let inject = Stage::new("inject").before("rendering").after("input");
    //let inject = Stage::new("injected").before("rendering").after("input");
    let after = Stage::new("after").after("inject").after("rendering");
    let sorted = evaluate(vec![rendering, input, inject, after]);

    for stage in sorted {
        dbg!(stage.name);
    }
}