use world::ecs::component::Component;

// A named component
#[derive(Component)]
pub struct Name(String);

impl Name {
    pub fn new<T: ToString>(obj: T) -> Self {
        Self(obj.to_string())
    }
}
