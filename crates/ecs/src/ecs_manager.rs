use crate::{system::SystemManager, ComponentManager, EntityManager};

// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    pub entitym: EntityManager,
    pub componentm: ComponentManager,
    pub systemm: SystemManager,
}
