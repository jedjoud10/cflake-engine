use world::ecs::component::Component;

// A tagged component
#[derive(Component)]
pub struct Tag(String);

impl Tag {
    pub fn new<T: ToString>(obj: T) -> Self {
        Self(obj.to_string())
    }
}
