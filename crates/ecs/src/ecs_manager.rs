use crate::{EntityManager, ComponentManager, system::SystemManager};

// The Entity Component System manager that will handle everything ECS related
pub struct ECSManager<T> {
    pub entitym: EntityManager, 
    pub componentm: ComponentManager,
    pub systemm: SystemManager<T>,
}

