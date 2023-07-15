use std::{any::TypeId, time::Duration};

use crate::{Caller, CallerId, Event, RegistrySortingError, Rule, StageError, StageId, SystemId, EventTimings};
use ahash::{AHashMap};

use lazy_static::lazy_static;
use petgraph::{Graph, visit::Topo};

// Reference point stages that we will use to insert more events into the registry
lazy_static! {
    pub static ref RESERVED_CALLER_TYPE_IDS: Vec<TypeId> = {
        // Custom reserved callers
        let init = TypeId::of::<crate::Init>();
        let update = TypeId::of::<crate::Update>();
        let shutdown = TypeId::of::<crate::Shutdown>();
        let tick = TypeId::of::<crate::Tick>();

        // Winit reserved callers
        let device = TypeId::of::<winit::event::DeviceEvent>();
        let window = TypeId::of::<winit::event::WindowEvent>();

        vec![init, update, shutdown, tick, device, window]
    };

    pub static ref RESERVED_CALLER_IDS: Vec<CallerId> = {
        vec![
            // Custom reserved callers
            super::fetch_caller_id::<crate::Init>(),
            super::fetch_caller_id::<crate::Update>(),
            super::fetch_caller_id::<crate::Shutdown>(),
            super::fetch_caller_id::<crate::Tick>(),

            // Winit reserved callers
            super::fetch_caller_id::<winit::event::DeviceEvent>(),
            super::fetch_caller_id::<winit::event::WindowEvent>()
        ]
    };


    pub static ref RESERVED_STAGE_IDS: Vec<StageId> = {
        let mut reserved: Vec<StageId> = Vec::new();

        // Create the reserved stage ID for all the user type callers
        let system = super::fetch_system_id(&crate::user);
        for caller in RESERVED_CALLER_IDS.iter() {
            reserved.push(super::combine_ids(&system, caller));
        }

        // Create the reserved stage ID for all the post user type callers
        let system = super::fetch_system_id(&crate::post_user);
        for caller in RESERVED_CALLER_IDS.iter() {
            reserved.push(super::combine_ids(&system, caller));
        }

        reserved
    };
}

// A registry is what will contain all the different stages, alongside the events
// Each type of event contains one registry associated with it
pub struct Registry<C: Caller + 'static> {
    // Name of the stage -> rules
    pub(super) map: AHashMap<StageId, Vec<Rule>>,

    // Name of the stage -> underlying event
    pub(super) events: Vec<(StageId, Box<C::DynFn>)>,

    // Keep last timings and total timings
    pub(super) timings_per_event: Vec<EventTimings<C>>,
    pub(super) timings_total: Duration,

    // Cached caller ID
    pub(super) caller: CallerId,
}

impl<C: Caller + 'static> Default for Registry<C> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            events: Default::default(),
            caller: super::fetch_caller_id::<C>(),
            timings_per_event: Default::default(),
            timings_total: Default::default(),
        }
    }
}

impl<C: Caller> Registry<C> {
    // Insert a new event that will be executed after the "user" stage and before the "post user" stage
    pub(crate) fn insert<ID>(
        &mut self,
        event: impl Event<C, ID> + 'static,
        system: SystemId,
    ) -> Result<&mut Vec<Rule>, StageError> {
        let rules = super::default_rules::<C>();
        let stage = super::combine_ids(&system, &self.caller);

        // We can only have one event per stage and one stage per event
        if self.map.contains_key(&stage) {
            Err(StageError::Overlapping)
        } else {
            // Check if the stage is valid
            if RESERVED_STAGE_IDS.contains(&stage) {
                return Err(StageError::InvalidName);
            }

            // Convert the stage and the event
            let boxed = event.boxed();

            // Insert the stage into the valid map
            let rules = self.map.entry(stage).or_insert(rules);

            // Then insert the event
            self.events.push((stage, boxed));
            self.timings_per_event.push(EventTimings::new(stage, C::persistent()));

            Ok(rules)
        }
    }

    // Sort all the events stored in the registry using the stages
    pub fn sort(&mut self) -> Result<(), RegistrySortingError> {
        let indices = sort(&self.map)?;

        // We do quite a considerable amount of mental trickery and mockery who are unfortunate enough to fall victim to our dever little trap of social teasing
        self.events.sort_by_key(|(x, _)| &indices[&x.system]);
        self.timings_per_event.sort_by_key(|x| &indices[&x.id().system]);

        log::debug!(
            "Sorted {} events for {} registry",
            self.events.len(),
            pretty_type_name::pretty_type_name_str(self.caller.name),
        );

        if !self.events.is_empty() {
            let slice = &self.events[..(self.events.len() - 1)];
            for (stage, _) in slice.iter() {
                log::debug!("├── {}", stage.system.name);
            }
            log::debug!("└── {}", self.events.last().unwrap().0.system.name);
        }

        // 3x POUNCES ON YOU UWU YOU'RE SO WARM
        Ok(())
    }

    // Execute all the events that are stored in this registry using specific arguments
    pub fn execute(&mut self, mut args: C::Args<'_, '_>) {
        let total = std::time::Instant::now();

        for (i, (_, event)) in self.events.iter_mut().enumerate() {
            let recorder = std::time::Instant::now();
            C::call(event, &mut args);
            self.timings_per_event[i].record(recorder.elapsed());
        }

        self.timings_total = total.elapsed();
    }

    // Get the per event timings and total timings
    pub fn timings(&self) -> (&[EventTimings<C>], Duration) {
        (&self.timings_per_event, self.timings_total)
    }
}

// Sort a hashmap containing multiple stage rules that depend upon each other
// This returns a hashmap containing the new indices of the sorted stages
fn sort(
    map: &AHashMap<StageId, Vec<Rule>>,
) -> Result<AHashMap<SystemId, usize>, RegistrySortingError> {
    let map = map.into_iter().collect::<Vec<_>>();
    let mut output = AHashMap::<SystemId, usize>::new();
    let mut graph = Graph::<SystemId, &Rule>::new();

    // Convert all stages into graph nodes
    let mut nodes = map.iter().map(|node| {
        (node.0.system, graph.add_node(node.0.system.clone()))
    }).collect::<AHashMap<_, _>>();
    
    // Insert the default user system
    let sid = crate::fetch_system_id(&crate::user);
    let user = graph.add_node(sid);
    nodes.insert(sid, user);
    
    // Insert the default post user system
    let sid = crate::fetch_system_id(&crate::post_user);
    let post_user = graph.add_node(sid);
    nodes.insert(sid, post_user);

    // Create the edges (rules) between the nodes (stages)
    for (node, rules) in map.iter() {
        
        // edges follow the direction of execution
        for rule in *rules {
            let this = nodes[&node.system];
            let reference = rule.reference();
            let reference = *nodes.get(&reference.system).ok_or_else(||
                RegistrySortingError::MissingStage(**node, reference)
            )?;

            match rule {
                // dir: a -> b.
                // dir: this -> reference
                Rule::Before(_) => graph.add_edge(this, reference, rule),

                // dir: a -> b.
                // dir: reference -> this
                Rule::After(_) => graph.add_edge(reference, this, rule),
            };
        }
    }

    // Topoligcally sort the graph (stage ordering)
    let mut topo = Topo::new(&graph);
    let mut counter = 0;
    while let Some(node) = topo.next(&graph) {
        let balls = nodes.iter().find(|x| *x.1 == node).unwrap();
        output.insert(balls.0.clone(), counter);
        counter += 1;
    }

    // If there are missing nodes then we must have a cylic reference
    if output.len() < map.len() {
        return Err(RegistrySortingError::GraphVisitMissingNodes);
    }

    Ok(output)
}
