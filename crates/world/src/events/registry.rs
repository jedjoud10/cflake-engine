use crate::{Caller, Event, RegistrySortingError, Rule, Stage, StageError, StageKey};
use ahash::{AHashMap, AHashSet};
use std::rc::Rc;

// Number of maximum iterations allowed before we detect a cyclic reference from within the rules
pub const CYCLIC_REFERENCE_RULES_THRESHOLD: usize = 8;

// Number of maximum iterations allowed before we detect a cyclic reference when recursing through the calc event
pub const CYCLIC_REFERENCE_THRESHOLD: usize = 50;

// Reference point stages that we will use to insert more events into the registry
pub const RESERVED: &[&str] = &["user", "post user"];

// A registry is what will contain all the different stages, alongside the events
// Each type of event contains one registry associated with it
pub struct Registry<C: Caller + 'static> {
    // Name of the stage -> rules
    pub(super) map: AHashMap<StageKey, Vec<Rule>>,

    // Name of the stage -> underlying event + duration
    pub(super) events: Vec<(StageKey, Box<C::DynFn>)>,

    // Incremented procedural name
    counter: u64,
}

impl<D: Caller + 'static> Default for Registry<D> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            events: Default::default(),
            counter: 0,
        }
    }
}

impl<C: Caller> Registry<C> {
    // Insert a new event that will be executed after the "user" stage and before the "post user" stage
    pub fn insert<ID>(&mut self, event: impl Event<C, ID>) {
        let name = Rc::from(format!("event-{}", self.counter));
        let stage = Stage::new(name).after("user").before("post user");
        self.counter += 1;
        self.insert_with::<ID>(event, stage).unwrap();
    }

    // Insert a new stage-event tuple into the registry (faillible)
    pub fn insert_with<ID>(
        &mut self,
        event: impl Event<C, ID>,
        stage: Stage,
    ) -> Result<(), StageError> {
        // We can only have one event per stage and one stage per event
        if self.map.contains_key(stage.name().as_ref()) {
            Err(StageError::Overlapping)
        } else {
            // Convert the stage and the event
            let stage = stage.validate()?;
            let boxed = event.boxed();

            // Insert the stage into the valid map
            let name = stage.name();
            self.map.insert(name.clone(), stage.into_rules());

            // Then insert the event
            self.events.push((name, boxed));
            Ok(())
        }
    }

    // Sort all the events stored in the registry using the stages
    pub fn sort(&mut self) -> Result<(), RegistrySortingError> {
        let _i = std::time::Instant::now();
        let indices = sort(&mut self.map)?;

        // We do quite a considerable amount of mental trickery and mockery who are unfortunate enough to fall victim to our dever little trap of social teasing
        self.events
            .sort_unstable_by(|(a, _), (b, _)| usize::cmp(&indices[a], &indices[b]));

        // 3x POUNCES ON YOU UWU YOU'RE SO WARM
        Ok(())
    }

    // Execute all the events that are stored in this registry using specific arguments
    pub fn execute(&mut self, mut args: C::Args<'_, '_>) {
        for (_, event) in self.events.iter_mut() {
            C::call(event, &mut args);
        }
    }
}

// Sort a hashmap containing multiple stage rules that depend upon each other
// This returns a hashmap containing the new indices of the sorted stages
fn sort(
    map: &AHashMap<StageKey, Vec<Rule>>,
) -> Result<AHashMap<StageKey, usize>, RegistrySortingError> {
    // Keep a hashmap containing the key -> indices and the global vector for our sorted stages (now converted to just rules)
    let mut map: AHashMap<StageKey, Vec<Rule>> = map.clone();

    // We might need to sort the keys to make sure they are deterministic
    let mut keys = map.keys().cloned().collect::<Vec<_>>();
    keys.sort();

    let mut indices = AHashMap::<StageKey, usize>::default();
    let mut vec = Vec::<Vec<Rule>>::default();

    // Insert the reserved stages, since we use them as reference points
    for reserved in RESERVED.iter() {
        vec.push(Vec::default());
        indices.insert(Rc::from(*reserved), vec.len() - 1);
    }

    // This event will add a current stage into the main vector and sort it according to it's rules
    fn calc(
        key: StageKey,
        indices: &mut AHashMap<StageKey, usize>,
        dedupped: &mut AHashMap<StageKey, Vec<Rule>>,
        current_tree: &mut AHashSet<StageKey>,
        vec: &mut Vec<Vec<Rule>>,
        iter: usize,
        caller: Option<StageKey>,
    ) -> Result<usize, RegistrySortingError> {
        // Check for a cyclic reference that might be caused when sorting the stages
        if iter > CYCLIC_REFERENCE_THRESHOLD {
            return Err(RegistrySortingError::CyclicReference);
        }

        if dedupped.contains_key(&key) {
            // We must insert the stage into the main vector
            let rules = dedupped.remove(&key).unwrap();
            current_tree.insert(key.clone());

            // Restrict the index of the stage based on it's rules
            let mut changed = true;
            let mut location = 0;
            let mut count = 0;

            // Check if we need to keep updating the location
            while changed {
                changed = false;

                // Restrict the current node using it's rules
                for rule in rules.iter() {
                    // Get the location of the parent stage
                    let parent = rule.parent();
                    let l = calc(
                        parent.clone(),
                        indices,
                        dedupped,
                        current_tree,
                        vec,
                        iter + 1,
                        Some(key.clone()),
                    )?;

                    match rule {
                        // Move the current stage BEFORE the parent stage
                        Rule::Before(_) => {
                            if location > l {
                                location = l.saturating_sub(1);
                                changed = true;
                            }
                        }

                        // Move the current stage AFTER the parent stage
                        Rule::After(_) => {
                            if location <= l {
                                location = l + 1;
                                changed = true;
                            }
                        }
                    }
                }

                // Check for a cyclic reference when constraining the stage
                count += 1;
                if count > CYCLIC_REFERENCE_RULES_THRESHOLD {
                    return Err(RegistrySortingError::CyclicRuleReference(key));
                }
            }

            // Insert the name -> index reference
            indices.insert(key.clone(), location);

            // Insert the new updated stage at it's correct location
            if location == vec.len() {
                vec.push(rules);
            } else if location < vec.len() {
                vec.insert(location, rules);
            } else if location > vec.len() {
                panic!("{} {}", location, vec.len());
            }

            // Update the indices of all the values that are after the current stage (since they were shifted to the right)
            for (name, i) in indices.iter_mut() {
                if *i >= location && name != &key {
                    *i += 1;
                }
            }

            Ok(location)
        } else {
            // We must check if the stage referenced by "called" is even valid
            if !indices.contains_key(&key) {
                if current_tree.contains(&key) {
                    return Err(RegistrySortingError::CyclicReference);
                } else {
                    return Err(RegistrySortingError::MissingStage(caller.unwrap(), key));
                }
            }

            // Fetch the cached location instead
            Ok(indices[&key])
        }
    }

    // Add the stages into the vector and start sorting them
    for key in keys {
        let mut tree = AHashSet::new();
        calc(key, &mut indices, &mut map, &mut tree, &mut vec, 0, None)?;
    }

    Ok(indices)
}
