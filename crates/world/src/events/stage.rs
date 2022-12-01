use crate::{Caller, System, World};
use std::any::{type_name, TypeId};
use winit::event::{DeviceEvent, WindowEvent};

// Stage ID that depicts the current location and ordering of a specific event and or stage
#[derive(
    Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Debug,
)]
pub struct StageId {
    pub caller: CallerId,
    pub system: SystemId,
}

// Single int to depict what caller we are using
#[derive(
    Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Debug,
)]
pub struct CallerId {
    // TODO: Use conditial compilation
    pub name: &'static str,

    // Init = 0
    // Update = 1
    // Shutdown = 2
    // Device event = 3
    // Window event = 4
    pub index: usize,

    // Used tp find said index
    pub id: TypeId,
}

// System id that contains the name and type ID of the system
#[derive(
    Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Debug,
)]
pub struct SystemId {
    // TODO: Use conditial compilation to disable this when we don't need to
    pub name: &'static str,

    pub id: TypeId,
}

// Combine two types of IDS
pub(crate) fn combine_ids(
    system: &SystemId,
    caller: &CallerId,
) -> StageId {
    StageId {
        caller: *caller,
        system: *system,
    }
}

// Get the caller ID of a specific caller type
pub(crate) fn fetch_caller_id<C: Caller>() -> CallerId {
    let id = TypeId::of::<C>();
    let index = crate::RESERVED_CALLER_TYPE_IDS
        .iter()
        .position(|current| *current == id)
        .unwrap();
    CallerId {
        name: type_name::<C>(),
        id: id,
        index,
    }
}

// Get the system ID of a specific system (simple generic function)
pub(crate) fn fetch_system_id<S: FnOnce(&mut System) + 'static>(
    _: &S,
) -> SystemId {
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
    // Get the current parent of the current strict node
    pub(super) fn parent(&self) -> StageId {
        match self {
            Rule::Before(p) => *p,
            Rule::After(p) => *p,
        }
    }
}

// Default user system and default events
pub fn user(system: &mut System) {
    system.insert_init(|world: &mut World| {});
    system.insert_update(|world: &mut World| {});
    system.insert_shutdown(|world: &mut World| {});
    system
        .insert_device(|world: &mut World, device: &DeviceEvent| {});
    system.insert_window(
        |world: &mut World, window: &mut WindowEvent| {},
    );
}

// Default post user system and default events
pub fn post_user(system: &mut System) {
    system.insert_init(|world: &mut World| {});
    system.insert_update(|world: &mut World| {});
    system.insert_shutdown(|world: &mut World| {});
    system
        .insert_device(|world: &mut World, device: &DeviceEvent| {});
    system.insert_window(
        |world: &mut World, window: &mut WindowEvent| {},
    );
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
