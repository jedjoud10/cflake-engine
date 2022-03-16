use crate::World;
use ecs::event::EcsEventSet;

// World events
// Currently only support ECS events
#[derive(Default)]
pub struct EventSet {
    pub ecs: EcsEventSet<World>,
}
