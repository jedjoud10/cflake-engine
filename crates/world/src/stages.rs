use ahash::{AHashMap};

use crate::StageError;

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Debug, Clone, Copy)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(Key),

    // This hints that the stage should be executed after other
    After(Key),
}

impl Rule {
    // Get the current parent of the current strict node
    fn parent(&self) -> Key {
        match self {
            Rule::Before(p) => p,
            Rule::After(p) => p,
        }
    }
}

// Stages are are a way for us to sort and prioritize certain events before others
#[derive(Clone)]
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // The direction of this rule compared to other rules
    rules: Vec<Rule>,
}

impl Stage {
    // This creates a stage builder so we can link some rules to the newly made stage
    pub fn builder() -> StageBuilder {
        StageBuilder(Stage {
            name: "",
            rules: Vec::default(),
        })
    }

    // Get the name of the current stage
    pub fn name(&self) -> Key {
        self.name
    }
    
    // Get the rules of the current stage
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

// A stage builder will help us create stages that have AT LEAST one rule associated with them
pub struct StageBuilder(Stage);

impl StageBuilder {
    // Update the name of the inner stage
    pub fn set_name(mut self, name: impl Into<Key>) -> Self {
        self.0.name = name.into();
        self
    }

    // Add a "before" rule to the current stage
    pub fn set_before(mut self, other: impl Into<Key>) -> Self {
        self.0.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a "after" rule to the current stage
    pub fn set_after(mut self, other: impl Into<Key>) -> Self {
        self.0.rules.push(Rule::After(other.into()));
        self
    }

    // Try to build the inner stage. If we have zero rules or if the name is empty, this will return None
    pub fn build(self) -> Option<Stage> {
        if self.0.name.is_empty() || self.0.rules.is_empty() {
            None
        } else {
            Some(self.0)
        }
    }
}

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// Number of maximum iterations allowed before we detect a cyclic reference when recursing through the calc function
const CYCLIC_REFERENCE_THRESHOLD: usize = 50;

// The name of the main execution start stage
const EXEC_STAGE_NAME: &str = "main";

// Calculate all the priority indices of the stages and sort them automatically
// This returns a hashmap containing the new indices of the sorted stages
pub(crate) fn sort(vec: Vec<Stage>) -> Result<AHashMap<Key, usize>, StageError> {
    // Convert the vector into a hashmap (this removes any duplicates)
    let mut dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    let keys: Vec<Key> = dedupped.keys().cloned().collect();

    // Keep a hashmap containing the key -> indices and the global vector for our sorted stages
    let mut indices = AHashMap::<Key, usize>::default();
    let mut vec = Vec::<Stage>::default();

    // Insert the startup Exec stage that will be the base of everything
    // Other stages can derive off of this by using multiple rules
    vec.push(Stage { name: EXEC_STAGE_NAME, rules: Vec::default() });
    indices.insert(EXEC_STAGE_NAME, 0);

    // This function will add a current stage into the main vector and sort it according to it's rules
    fn calc(key: Key, indices: &mut AHashMap<Key, usize>, dedupped: &mut AHashMap<Key, Stage>, vec: &mut Vec<Stage>, iter: usize, caller: Option<Key>) -> Result<usize, StageError> {
        // Check for a cyclic reference that might be caused when sorting the stages
        if iter > CYCLIC_REFERENCE_THRESHOLD {
            return Err(StageError::CyclicReference);
        }
        
        if dedupped.contains_key(key) {
            // We must insert the stage into the main vector
            let stage = dedupped.remove(key).unwrap();
            let rules = stage.rules();

            // Restrict the index of the stage based on it's rules
            let mut changed = true;
            let mut location = 0;
            let mut count = 0;

            // Check if we need to keep updating the location
            while changed {
                changed = false;

                // Restrict the current node using it's rules
                for rule in rules {
                    // Get the location of the parent stage
                    let parent = rule.parent();
                    let parent_location = calc(parent, indices, dedupped, vec, iter + 1, Some(key))?;

                    match rule {                        
                        // Move the current stage BEFORE the parent stage
                        Rule::Before(_) => {
                            if location > parent_location {
                                location = parent_location - 1;
                                changed = true;
                            }
                        },

                        // Move the current stage AFTER the parent stage
                        Rule::After(_) => {
                            if location <= parent_location {
                                location = parent_location + 1;
                                changed = true;
                            }
                        },
                    }
                }
                
                // Check for a cyclic reference when constraining the stage
                count += 1;
                if count > CYCLIC_REFERENCE_RULES_THRESHOLD {
                    return Err(StageError::CyclicRuleReference(key));
                }
            }

            // Insert the new updated stage at it's correct location
            indices.insert(stage.name, vec.len());
            vec.insert(location, stage);

            // Updated indices (slow!)
            for (i, stage) in vec.iter_mut().enumerate() {
                *indices.get_mut(stage.name).unwrap() = i;
            }

            Ok(location)
        } else {
            // We must check if the stage referenced by "called" is even valid
            if !indices.contains_key(key) {
                return Err(StageError::MissingStage(caller.unwrap(), key));
            }

            // Fetch the cached location instead
            Ok(indices[key])
        }
    }

    // Add the stages into the vector and start sorting them
    for key in keys {
        calc(key, &mut indices, &mut dedupped, &mut vec, 0, None)?;
    }

    // Remove the "exec" stage since it was just there to act as a reference point
    vec.remove(indices[EXEC_STAGE_NAME]);
    indices.remove(EXEC_STAGE_NAME);

    Ok(indices)
}