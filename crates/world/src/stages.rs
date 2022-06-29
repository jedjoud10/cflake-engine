use std::ops::{Bound, RangeBounds, RangeFull};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A restriction is some sort of rule that depicts the arrangement and the location of the stages
enum Restriction {
    Before(Key),
    After(Key)
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Restrictions that apply to this stage
    restrictions: Vec<Restriction>,
}

impl Stage {
    // Create a stage that will execute anytime, since it has no restrictions
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), restrictions: Default::default() }
    }

    // Add a restriction to this stage hinting it that it should execute before other
    pub fn before(mut self, other: impl Into<Key>) -> Self {
        self.restrictions.push(Restriction::Before(other.into()));
        self
    }

    // Add a restriction to this stage hinting it that it should execute after other
    pub fn after(mut self, other: impl Into<Key>) -> Self {
        self.restrictions.push(Restriction::After(other.into()));
        self
    }

    // Check if the stage references the "other" stage in any way 
    fn is_parent(&self, other: Key) -> bool {
        self.restrictions.iter().any(|r| match *r {
            Restriction::Before(k) => k == other,
            Restriction::After(k) => k == other,
        })
    }
}

// Calculate all the priority indices from a set of stages
pub fn evaluate(map: AHashMap<Key, Stage>) {
    // This contains the priorities associated with each stage
    let mut priorities: AHashMap<Key, i32> = AHashMap::new();

    // These are the stages that we must evalute for currently
    let mut eval: AHashSet<Key> = AHashSet::new();

    // Evaluate the stages that have no restriction (p = 0)
    for (_, stage) in map.iter().filter(|(_, stage)| stage.restrictions.is_empty()) {
        eval.insert(stage.name);
    }

    // Repeatedly evalute the stages
    loop {
        for key in eval.drain() {
            // Calculate the priority index for the current node
            let stage = &map[key];
            if stage.restrictions.is_empty() {
                // If we have no restrictions, the priority index is 0
                priorities.insert(key, 0);
            } else {
                // Calculate the priority index based on restrictions
                for restriction in stage.restrictions.iter() {
                    let parent = match restriction {
                        Restriction::Before(parent) => &map[parent],
                        Restriction::After(parent) => &map[parent],
                    }
                }  
            }
        }
    }

    Stage::new(rendering).after("input").before("internal")

    // Evalue the stages that use our current evaluted nodes as parents (down a level in tree)
    for stage in map.values().filter(|stage| stage.is_parent( ))
}

#[test]
fn test() {
    let main = Stage::new("main");
    let after = Stage::new("after").after("main");
    let before = Stage::new("before").before("main");
}
