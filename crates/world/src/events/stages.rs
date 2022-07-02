/*
use ahash::AHashMap;
use crate::StageError;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Clone)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(String),

    // This hints that the stage should be executed after other
    After(String),
}

impl Rule {
    // Get the current parent of the current strict node
    fn parent(&self) -> &str {
        match self {
            Rule::Before(p) => &p,
            Rule::After(p) => &p,
        }
    }
}

// Stages are are a way for us to sort and prioritize certain events before others
// Stages will be converted to Nodes whenever they get inserted into a pipeline
#[derive(Clone)]
pub struct Stage {
    // The user defined name of the current stage
    name: String,

    // The direction of this rule compared to other rules
    rules: Vec<Rule>,

    // The pipeline where we will insert this stage
    pipeline: String,
}

impl Stage {
    // Create a new empty stage with no rules and no pipeline (invalid)
    pub fn new(name: impl Into<String>, pipeline: impl Into<String>) -> Self {
        Self { name: name.into(), pipeline: pipeline.into(), rules: Vec::new()  }
    }

    // Add a "before" rule to the current stage
    pub fn set_before(mut self, other: impl Into<String>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a "after" rule to the current stage
    pub fn set_after(mut self, other: impl Into<String>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self
    }

    // Set the pipeline where the stage should be inserted to
    pub fn set_pipeline(mut self, pipeline: impl Into<String>) -> Self {
        self.pipeline = pipeline.into();
        self
    } 
}

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// Number of maximum iterations allowed before we detect a cyclic reference when recursing through the calc function
const CYCLIC_REFERENCE_THRESHOLD: usize = 50;

// The name of the main execution start stage
const EXEC_STAGE_NAME: &str = "main";

// A pipeline is what will contain all the different stages
// Multiple types of events can have different pipelines, however, we must assume that the pipelines have no dependencies upon each other
#[derive(Default)]
pub(super) struct Pipeline(AHashMap<String, Stage>);

/*
// Calculate all the priority indices of the stages and sort them automatically
// This returns a hashmap containing the new indices of the sorted stages
pub(crate) fn sort(vec: Vec<Stage>) -> Result<AHashMap<Key, usize>, StageError> {
    // Convert the vector into a hashmap (this removes any duplicates)
    let mut dedupped: AHashMap<Key, Stage> =
        AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    let keys: Vec<Key> = dedupped.keys().cloned().collect();

    // Keep a hashmap containing the key -> indices and the global vector for our sorted stages
    let mut indices = AHashMap::<Key, usize>::default();
    let mut vec = Vec::<Stage>::default();

    // Insert the startup Exec stage that will be the base of everything
    // Other stages can derive off of this by using multiple rules
    vec.push(Stage {
        name: EXEC_STAGE_NAME,
        rules: Vec::default(),
    });
    indices.insert(EXEC_STAGE_NAME, 0);

    // This function will add a current stage into the main vector and sort it according to it's rules
    fn calc(
        key: Key,
        indices: &mut AHashMap<Key, usize>,
        dedupped: &mut AHashMap<Key, Stage>,
        vec: &mut Vec<Stage>,
        iter: usize,
        caller: Option<Key>,
    ) -> Result<usize, StageError> {
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
                    let parent_location =
                        calc(parent, indices, dedupped, vec, iter + 1, Some(key))?;

                    match rule {
                        // Move the current stage BEFORE the parent stage
                        Rule::Before(_) => {
                            if location > parent_location {
                                location = parent_location - 1;
                                changed = true;
                            }
                        }

                        // Move the current stage AFTER the parent stage
                        Rule::After(_) => {
                            if location <= parent_location {
                                location = parent_location + 1;
                                changed = true;
                            }
                        }
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
*/
*/