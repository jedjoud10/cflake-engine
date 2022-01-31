use std::collections::HashMap;

use fonts::FontManager;

use crate::Root;

// The UI manager, it can contain multiple UI roots, and switch between them
#[derive(Default)]
pub struct UIManager {
    pub roots: HashMap<String, Root>,
    pub font_manager: FontManager,
}

// Actually UI functions
impl UIManager {
    // Get the root with the corresponding name
    pub fn get_root(&self, name: &str) -> &Root {
        self.roots.get(name).unwrap()
    }
    // Get the root with the corresponding name mutably
    pub fn get_root_mut(&mut self, name: &str) -> &Root {
        self.roots.get_mut(name).unwrap()
    }
    // Add a root to the manager
    pub fn add_root(&mut self, root_name: &str, root: Root) {
        self.roots.insert(root_name.to_string(), root);
    }
}
