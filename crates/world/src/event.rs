use ecs::event::EcsEventSet;
use crate::World;

// World events
// Currently only support ECS events
#[derive(Default)]
pub struct EventSet {
    pub ecs: EcsEventSet<World>,
}