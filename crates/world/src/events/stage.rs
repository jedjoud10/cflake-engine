use crate::{Caller, System, World};
use std::any::{type_name, TypeId};
use winit::event::{DeviceEvent, WindowEvent};

// Stage ID that depicts the current location and ordering of a specific event and or stage
#[derive(Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct StageId {
    pub caller: CallerId,
    pub system: SystemId,
}

// Single int to depict what caller we are using
#[derive(Clone, Copy, Eq, Debug)]
pub struct CallerId {
    pub name: &'static str,
    pub id: TypeId,
}

impl std::hash::Hash for CallerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for CallerId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for CallerId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for CallerId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

// System id that contains the name and type ID of the system
#[derive(Clone, Copy, Eq, Debug)]
pub struct SystemId {
    pub name: &'static str,
    pub id: TypeId,
}

impl std::hash::Hash for SystemId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for SystemId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for SystemId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for SystemId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

// Combine two types of IDS
pub fn combine_ids(system: &SystemId, caller: &CallerId) -> StageId {
    StageId {
        caller: *caller,
        system: *system,
    }
}

// Get the caller ID of a specific caller type
pub fn fetch_caller_id<C: Caller>() -> CallerId {
    let id = TypeId::of::<C>();
    crate::RESERVED_CALLER_TYPE_IDS
        .iter()
        .position(|current| *current == id)
        .unwrap();
    CallerId {
        name: type_name::<C>(),
        id,
    }
}

// Get the system ID of a specific system (simple generic function)
pub fn fetch_system_id<S: FnOnce(&mut System) + 'static>(_: &S) -> SystemId {
    SystemId {
        name: type_name::<S>(),
        id: TypeId::of::<S>(),
    }
}

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Clone, Debug)]
pub enum Rule {
    // This hints that the stage should be executed before other
    Before(StageId),

    // This hints that the stage should be executed after other
    After(StageId),
}

impl Rule {
    // Get the node this rule is referencing
    pub(super) fn reference(&self) -> StageId {
        match self {
            Rule::Before(p) => *p,
            Rule::After(p) => *p,
        }
    }
}

// Default user system and default events
pub fn user(system: &mut System) {
    system.insert_init(|_world: &mut World| {});
    system.insert_update(|_world: &mut World| {});
    system.insert_shutdown(|_world: &mut World| {});
    system.insert_tick(|_world: &mut World| {});
    system.insert_device(|_world: &mut World, _device: &DeviceEvent| {});
    system.insert_window(|_world: &mut World, _window: &mut WindowEvent| {});
}

// Default post user system and default events
pub fn post_user(system: &mut System) {
    system.insert_init(|_world: &mut World| {});
    system.insert_update(|_world: &mut World| {});
    system.insert_tick(|_world: &mut World| {});
    system.insert_shutdown(|_world: &mut World| {});
    system.insert_device(|_world: &mut World, _device: &DeviceEvent| {});
    system.insert_window(|_world: &mut World, _window: &mut WindowEvent| {});
}

// Create the default rules for a default node
pub(super) fn default_rules<C: Caller>() -> Vec<Rule> {
    let caller = fetch_caller_id::<C>();

    // Create the default after rule
    let system = fetch_system_id(&user);
    let stage = combine_ids(&system, &caller);
    let after = Rule::After(stage);

    // Create the default before rule
    let system = fetch_system_id(&post_user);
    let stage = combine_ids(&system, &caller);
    let before = Rule::Before(stage);

    // Combine both rules
    vec![before, after]
}
