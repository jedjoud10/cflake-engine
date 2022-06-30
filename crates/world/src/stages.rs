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

// How much of a gap to we leave for other events to slot in
const GAP: i32 = 4096;

// Number of maximum calculations allowed before we detect a cyclic references
const CYCLIC_REFERENCE_THRESHOLD: usize = 8;

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // Convert the vector into a hashmap (this removes any duplicates)
    let dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    
    // This map contains all the priority indices that we will need for sorting
    let mut priorities: AHashMap<Key, i32> = AHashMap::default();

    // Calculate the priority index and the depth level for a single node
    fn calc(key: Key, dedupped: &AHashMap<Key, Stage>, priorities: &mut AHashMap<Key, i32>, iterdepth: usize) -> i32 {
        // Check for cyclic references, and panic if found
        if iterdepth > CYCLIC_REFERENCE_THRESHOLD {
            panic!("Cyclic reference detected")
        }

        // Check if it has any rules associated with it
        let stage = &dedupped[key];
        let rules = stage.rules.as_slice();

        // Get the cached priorities if needed
        if priorities.contains_key(key) {
            println!("Fetching cached priority. {key} {}", priorities[key]);
            return priorities[key];
        } else {            
            println!("Calculating priority for stage: {}", key);

            // Caclulate the stage priority manually now
            if !rules.is_empty() {
                // Restrict the current node priority using the rules
                let mut priority = 0;
                let mut changed = true;
                let mut i = 0;
                let mut count = 0;

                while changed {
                    changed = false;
                    match rules[i] {
                        Rule::Before(parent) => {
                            let pp = calc(parent, dedupped, priorities, iterdepth + 1);
                            println!("parent: {parent}, pp: {pp}");
                            if priority >= pp {
                                priority = pp - GAP;
                                changed = true;
                            }
                        },
                        Rule::After(parent) => {
                            let pp = calc(parent, dedupped, priorities, iterdepth + 1);
                            if priority <= pp {
                                priority = pp + GAP;
                                changed = true;
                            }
                        },
                    }

                    i = (i + 1) % rules.len();
                    count += 1;

                    if count > CYCLIC_REFERENCE_RULES_THRESHOLD {
                        panic!("Rule cyclic reference detected");
                    }
                }

                // Update the priority and depth values
                priorities.insert(key, priority);
            } else {
                println!("Current stage is a free stage, priority = 0");
                // Free nodes have a priority of zero
                priorities.insert(key, 0);
            }

            // Fetch the new priority again
            println!("p[{key}] = {}", priorities[key]);
            priorities[key]
        }
    }

    // Calculate the priorities of all stages
    for stage in dedupped.values() {
        calc(stage.name, &dedupped, &mut priorities, 0);
    }

    // Convert all the maps to a singular vector, and sort them
    let mut vec: Vec<(Stage, i32)> = Vec::new();

    for (key, stage) in dedupped.into_iter() {
        let priority = priorities[key];
        vec.push((stage, priority));
    }

    // Sort the vector now, then convert it to a vector to without priorities
    vec.sort_unstable_by(|(_, a), (_, b)| i32::cmp(a, b));
    
    // Convert back (slow!)
    vec.into_iter().map(|(stage, _)| stage).collect()
}


#[test]
fn test() {
    let node1 = Stage::new("main");
    let input = Stage::new("input").before("main");
    let rendering = Stage::new("rendering").after("input");
    let inject = Stage::new("injected").before("rendering").after("input");
    let after = Stage::new("after").after("injected");
    let sorted = evaluate(vec![after, node1, input, rendering, inject]);

    for stage in sorted {
        dbg!(stage.name);
    }
}