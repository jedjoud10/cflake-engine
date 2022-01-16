use crate::impl_component;

// Create some default components

// A name component that can be added to named entities
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

// A tag component that can be added to entities that contain some sort of "Tag" We can then search for entities with the same tag
pub struct Tagged {
    pub tag: String,
}

impl Tagged {
    pub fn new(tag: &str) -> Self {
        Self { tag: tag.to_string() }
    }
}

// Load state for entities
pub enum LoadState {
    Loaded,
    Unloaded,
}

impl_component!(Name);
impl_component!(Tagged);
impl_component!(LoadState);
