use ahash::AHashMap;
use petgraph::{Graph, stable_graph::NodeIndex, visit::Topo};
use thiserror::Error;
use winit::event::{WindowEvent, DeviceEvent};
use crate::{events::{Event, Tick, Shutdown, Init, Update}, world::World};
use std::{marker::PhantomData, any::TypeId};
use super::{System, InjectionOrder, InjectionRule, SystemTimings};

#[derive(Default)]
/// Contains multiple registries for each type of hookable event
pub struct Registries {
    /// Init event registry
    pub init: Registry<Init>,
    
    /// Update event registry
    pub update: Registry<Update>,
    
    /// Shutdown event registry
    pub shutdown: Registry<Shutdown>,
    
    /// Tick event registry
    pub tick: Registry<Tick>,
    
    /// Window event registry
    pub window_event: Registry<WindowEvent>,

    /// Device event registry
    pub device_event: Registry<DeviceEvent>,
}

/// A registry is what will contain all the different stages, and their appropriate systems.
/// Stages are executed sequentially, although the systems within them are executed in parallel (if possible).
/// Each type of event contains one registry associated with it.
pub struct Registry<E: Event> {
    // Injection rules for all given systems (sorted and unsorted)
    rules: AHashMap<TypeId, Vec<InjectionRule>>,

    // Unsorted events that cannot be executed
    unsorted: AHashMap<TypeId, Box<dyn System<E>>>,

    // Sorted events are events that we can execute
    sorted: Vec<(Box<dyn System<E>>, u32)>,

    // System timings for the sorted events
    _phantom: PhantomData<E>,
}

impl<E: Event> Default for Registry<E> {
    fn default() -> Self {
        Self {
            sorted: Default::default(),
            unsorted: Default::default(),
            rules: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<E: Event> Registry<E> {
    /// Insert a new system into the registry.
    /// Returns an [InjectionOrder<E>] that you can use to set the system's execution stage.
    /// Automatically sets the system to execute after [super::pre_user] and before [super::post_user].
    pub fn insert<S: System<E>>(&mut self, system: S) -> InjectionOrder<E> {
        let type_id = TypeId::of::<S>();
        
        if self.rules.contains_key(&type_id) {
            log::warn!("Replaced system {} since it was already present", std::any::type_name::<S>());
        }
        
        self.rules.insert(type_id, Vec::new());
        self.unsorted.insert(type_id, Box::new(system));
        let rules = self.rules.get_mut(&type_id).unwrap();
        rules.push(InjectionRule::After(super::type_id_of_val(super::pre_user::<E>)));
        rules.push(InjectionRule::Before(super::type_id_of_val(super::post_user::<E>)));
        
        InjectionOrder {
            rules: rules,
            default: true,
            _phantom: PhantomData,
        }
    }

    /// Sort all the systems stored in the registry using their rules.
    pub fn sort(&mut self) -> Result<(), RegistrySortingError> {
        // Create a topologically sorted graph that will take acount the rules of all systems
        let mut graph = Graph::<TypeId, ()>::new();

        // Convert all systems into graph nodes
        let mut nodes = self.rules.iter().map(|(type_id, rules)| 
            (*type_id, (graph.add_node(*type_id), rules.as_slice()))
        ).collect::<AHashMap<TypeId, (NodeIndex, &[InjectionRule])>>();
    
        // Insert the default pre user system
        let id = super::type_id_of_val(super::pre_user::<E>);
        let index = graph.add_node(id);
        nodes.insert(id, (index, &[]));
    
        // Insert the default post user system
        let id = super::type_id_of_val(super::post_user::<E>);
        let index = graph.add_node(id);
        nodes.insert(id, (index, &[]));
    
        // Create the edges (rules) between the nodes (systems)
        for (this_type_id, (this, rules)) in nodes.iter().filter(|(_, (_, r))| !r.is_empty()) {
            for rule in *rules {
                let reference = match rule {
                    InjectionRule::Before(x) => x,
                    InjectionRule::After(x) => x,
                };

                let (reference, _) = *nodes
                    .get(reference)
                    .ok_or_else(|| {
                        let referencer = self.unsorted[this_type_id].name();
                        let referencee = self.unsorted[reference].name();
                        RegistrySortingError::MissingSystem(referencer, referencee)
                    })?;
    
                match rule {
                    // dir: a -> b.
                    // dir: this -> reference
                    InjectionRule::Before(_) => graph.add_edge(*this, reference, ()),
    
                    // dir: a -> b.
                    // dir: reference -> this
                    InjectionRule::After(_) => graph.add_edge(reference, *this, ()),
                };
            }

            log::trace!("{}: {} rules", self.unsorted[this_type_id].name(), rules.len());
        }
    
        // Topoligcally sort the graph (stage ordering)
        let mut topo = Topo::new(&graph);
        let mut counter = 0;

        while let Some(node) = topo.next(&graph) {
            let type_id = graph.node_weight(node).unwrap();
            
            if let Some(system) = self.unsorted.remove(type_id) {
                self.sorted.push((system, counter))
            }

            counter += 1;
        }

        // We do quite a considerable amount of mental trickery and mockery who are unfortunate enough to fall victim to our dever little trap of social teasing
        self.sorted.sort_by_key(|(_, int)| *int);

        // If there are missing nodes then we must have a cylic reference
        if self.sorted.len() < self.rules.len() {
            log::error!("There are {} sorted systems, but there must be {}", self.sorted.len(), self.rules.len());
            log::trace!("available systems: ");
            for (system, _) in self.sorted.iter() {
                log::trace!("{}", system.name());
            }

            return Err(RegistrySortingError::GraphVisitMissingNodes);
        }    

        log::debug!(
            "Sorted {} systems for {} registry",
            self.rules.len(),
            std::any::type_name::<E>(),
        );

        // Show a debug GUI of the systems
        if !self.sorted.is_empty() {
            let slice = &self.sorted[..(self.sorted.len() - 1)];
            for (stage, _) in slice.iter() {
                log::debug!("├── {}", stage.name());
            }
            log::debug!("└── {}", self.sorted.last().unwrap().0.name());
        }
        
        Ok(())
    }

    /// Get an immutable slice of the internal sorted systems and their offsets
    pub fn sorted_systems(&self) -> &[(Box<dyn System<E>>, u32)] {
        &self.sorted
    }

    /// Execute all the sorted systems in this registry
    pub fn execute(&mut self, world: &mut World, event: &E) {
        for (system, _) in self.sorted.iter_mut() {
            system.execute(world, event)
        }
    }
}

/// Error that gets thrown whenever we fail to sort the systems
#[derive(Error, Debug)]
pub enum RegistrySortingError {
    /// Error while parsing Graph. Possibly due to cyclic reference / rules
    #[error("Error while parsing Graph. Possibly due to cyclic reference / rules")]
    GraphVisitMissingNodes,

    /// Tried referencing a non existant system
    #[error("Stage '{0}' tried to reference system '{1}', but the latter system does not exist")]
    MissingSystem(&'static str, &'static str),
}