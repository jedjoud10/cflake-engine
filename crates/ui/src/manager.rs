use std::{collections::HashMap, sync::{Mutex, Arc}};

use fonts::FontManager;

use crate::Root;

// The UI manager, it can contain multiple UI roots, and switch between them
pub struct UIManager {
    pub roots: Arc<Mutex<HashMap<String, Root>>>,
    pub font_manager: FontManager,
}

impl Default for UIManager {
    fn default() -> Self {
        // Create a default root
        let default_root = Root::default();
        let mut roots = HashMap::new();
        roots.insert("default".to_string(), default_root);
        Self { roots: Arc::new(Mutex::new(roots)), font_manager: Default::default() }
    }
}

// Actually UI functions
impl UIManager {
    // Update the default root
    pub fn update_default<F: FnMut(&mut FontManager, &mut Root)>(&mut self, mut update_function: F) {
        // Get the default root and update
        let cloned_ = self.roots.clone();
        let mut lock = cloned_.lock().unwrap();
        let root = lock.get_mut("default").unwrap();
        update_function(&mut self.font_manager, root);
    }
    // Update a root using it's name as identifier
    pub fn update_root<F: FnMut(&mut FontManager, &mut Root)>(&mut self, root_name: &str, mut update_function: F) {
        // Get the root and update
        let cloned_ = self.roots.clone();
        let mut lock = cloned_.lock().unwrap();
        let root = lock.get_mut(root_name).unwrap();
        update_function(&mut self.font_manager, root);
    }
    // Add a root to the manager
    pub fn add_root(&mut self, root_name: &str, root: Root) {
        let mut lock = self.roots.lock().unwrap();
        lock.insert(root_name.to_string(), root);
    }
}
