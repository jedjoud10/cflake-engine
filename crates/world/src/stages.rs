use std::ops::{Bound, RangeBounds, RangeFull};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Debug, Clone, Copy)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(Key),

    // This hints that the stage should be executed after other
    After(Key),

    // This hints that the stage should be executed after A but before B
    Bounded(Key, Key),
}

impl Rule {
    // Create the before rule
    pub fn before(other: Key) -> Self {
        Self::Before(other)
    }
    
    // Create the after rule
    pub fn after(other: Key) -> Self {
        Self::After(other)
    }

    // Check if the rules contain any of these names
    fn contains(&self, name: Key) -> bool {
        match *self {
            Rule::Before(p) => p == name,
            Rule::After(p) => p == name,
            Rule::Bounded(a, b) => a == name || b == name,
        }
    }
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Stages can only have one rule (max)
    rule: Option<Rule>,
}

impl Stage {
    // Create a stage with no rules whatsoever
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rule: None }
    }

    // Create a stage with a single rule (before or after)
    pub fn strict(name: impl Into<Key> + Clone, rule: Rule) -> Self {
        if rule.contains(name.clone().into()) {
            panic!()
        }

        Self { name: name.into(), rule: Some(rule) }
    }
}

// How much of a gap to we leave for other events to slot in
const GAP: i32 = 4096;

// Number of maximum calculations allowed before we detect a cyclic references
const CYCLIC_REFERENCE_THRESHOLD: usize = 8;

// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // Convert the vector into a hashmap (this removes any duplicates)
    let dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    
    // This map contains all the priority indices that we will need for sorting
    let mut priorities: AHashMap<Key, i32> = AHashMap::default();
    let mut depths: AHashMap<Key, u32> = AHashMap::default();

    // Calculate the priority index and the depth level for a single node
    fn calc(key: Key, dedupped: &AHashMap<Key, Stage>, priorities: &mut AHashMap<Key, i32>, depths: &mut AHashMap<Key, u32>, iterdepth: usize) -> (i32, u32) {
        // Check for cyclic references, and panic if found
        if iterdepth > CYCLIC_REFERENCE_THRESHOLD {
            panic!("Cyclic reference found")
        }

        // Check if it has any rules associated with it
        let stage = &dedupped[key];
        let rule = stage.rule.as_ref();

        // Get the cached priorities if needed
        if priorities.contains_key(key) {
            println!("Fetching cached priority. {}", priorities[key]);
            return (priorities[key], depths[key]);
        } else {            
            println!("Calculating priority for stage: {}", key);
            // Caclulate the stage priority manually now
            if let Some(rule) = rule {
                // Parent data rules
                // I32 indicates the priority
                // U32 indicates the depth
                enum RuleData {
                    Before((i32, u32)),
                    After((i32, u32)),
                    Bounded {
                        parent_a: (i32, u32),
                        parent_b: (i32, u32),
                    },
                }

                // Fetch the priorities of the parents
                let pp = match rule {
                    Rule::Before(p) => RuleData::Before(calc(p, dedupped, priorities, depths, iterdepth+1)),
                    Rule::After(p) => RuleData::After(calc(p, dedupped, priorities, depths, iterdepth+1)),
                    Rule::Bounded(a, b) => RuleData::Bounded {
                        parent_a: calc(a, dedupped, priorities, depths, iterdepth+1),
                        parent_b: calc(b, dedupped, priorities, depths, iterdepth+1),
                    },
                };

                // Check if the RuleData::Bounded is valid (B > A)
                if let RuleData::Bounded { parent_a, parent_b } = pp {
                    assert!(parent_b.0 > parent_a.0, "Bounds not valid");
                }
                
                // Calcualte priority AND depth at the same time for the current node
                let (priority, depth) = match pp {
                    RuleData::Before((priority, depth)) => {
                        (priority - (GAP / (depth + 1) as i32), depth + 1)
                    },
                    RuleData::After((priority, depth)) => {
                        (priority + (GAP / (depth + 1) as i32), depth + 1)
                    },
                    RuleData::Bounded { parent_a: (pa, da), parent_b: (pb, db) } => {
                        ((pa + pb) / 2, db.max(da) + 1)
                    },
                };

                // Update the values internally
                priorities.insert(key, priority);
                depths.insert(key, depth);
            } else {
                println!("Current stage is a free stage, priority = 0");
                // Free nodes have a priority of zero
                priorities.insert(key, 0);
                depths.insert(key, 0);
            }

            // Fetch the new priority and updated depth again
            println!("p[{key}] = {}", priorities[key]);
            println!("d[{key}] = {}", depths[key]);
            (priorities[key], depths[key])
        }
    }

    // Calculate the priorities of all stages
    for stage in dedupped.values() {
        calc(stage.name, &dedupped, &mut priorities, &mut depths, 0);
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
    let input = Stage::strict("input", Rule::Before("main"));
    let rendering = Stage::strict("rendering", Rule::After("input"));
    let inject = Stage::strict("injected", Rule::Bounded("input", "rendering"));
    let sorted = evaluate(vec![node1, input, rendering, inject]);

    for stage in sorted {
        dbg!(stage.name);
    }
}