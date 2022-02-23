use crate::component::Component;
// A name component that can be added to named entities
#[derive(Component)]
pub struct Name {
    pub name: String,
}

impl Default for Name {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
        }
    }
}

impl Name {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

// A tag component that can be added to entities that contain some sort of "Tag" We can then search for entities with the same tag
#[derive(Component)]
pub struct Tagged {
    pub tag: String,
}

impl Default for Tagged {
    fn default() -> Self {
        Self {
            tag: "Untagged".to_string(),
        }
    }
}

impl Tagged {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }
}
