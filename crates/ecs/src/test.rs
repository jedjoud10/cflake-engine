#[cfg(test)]
pub mod test {
    use crate::{ECSManager, Entity, ComponentLinkingGroup};

    pub struct TestComponent {
        pub val: f32,
    }
    crate::impl_component!(TestComponent);

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        let mut ecs = ECSManager::default();
        let mut group = ComponentLinkingGroup::new();
        group.link(TestComponent { val: 0.0 }).unwrap();
        let entity = Entity::new();
        let id = ecs.add_entity(Entity::new());
        ecs.add_component_group(id, group).unwrap();
    }
}