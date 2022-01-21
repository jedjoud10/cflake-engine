use cflake_engine::*;
use window::{start};
fn main() {
    // Load up the engine
    start("DevJed", "DevGame", preload_assets, init);
}
pub fn preload_assets() {
    // -----Pre-load the game assets here-----
}
pub fn init(mut write: window::core::WriteContext, task_sender: window::core::TaskSenderContext) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ecs::entity::ComponentLinkingGroup::new();
    group.link(defaults::components::Camera::new(90.0, 0.5, 1000.0)).unwrap();
    group.link_default::<defaults::components::Transform>().unwrap();
    let entity = ecs::entity::Entity::new();
    let id = ecs::entity::EntityID::new(&mut write.ecs);
    ecs::tasks::add_entity(&task_sender, entity, id, group).unwrap();
}
