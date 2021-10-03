use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use fonts::FontManager;

use crate::{element::ButtonState, element::ElementType, Root};

// The UI manager, it can contain multiple UI roots, and switch between them
#[derive(Default)]
pub struct UIManager {
    pub roots: HashMap<String, Root>,
    pub font_manager: FontManager,
}

// Actually UI functions
impl UIManager {
    // Check if we have a default root set
    pub fn default_root_valid(&self) -> bool {
        self.roots.contains_key("default")
    }
    // Get the default root
    pub fn get_default_root(&self) -> &Root {
        self.roots.get("default").unwrap()
    }
    pub fn get_default_root_mut(&mut self) -> &mut Root {
        self.roots.get_mut("default").unwrap()
    }
    // Set the default UI root that will be drawn on the screen by default
    pub fn set_default_root(&mut self, root: Root) {
        self.roots.entry("default".to_string()).or_insert(root);
    }
    // Get the root with the corresponding name
    pub fn get_root_mut(&mut self, name: &str) -> &mut Root {
        self.roots.get_mut(name).unwrap()
    }
    // Add a root to the manager
    pub fn add_root(&mut self, root_name: &str, root: Root) {
        self.roots.insert(root_name.to_string(), root);
    }
}
