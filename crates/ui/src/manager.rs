use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use fonts::FontManager;

use crate::Root;

// The UI manager, it can contain multiple UI roots, and switch between them
#[derive(Default)]
pub struct UIManager {
    pub roots: Arc<Mutex<HashMap<String, Root>>>,
    pub font_manager: FontManager,
}

// Actually UI functions
impl UIManager {
    // Update a root
    pub fn update_root(&self, root_name: &str, function: impl FnOnce(&mut Root)) -> Option<()> {
        let mut lock = self.roots.lock().ok()?;
        function(lock.get_mut(root_name)?);
        Some(())
    }
    // Add a root to the manager
    pub fn add_root(&mut self, root_name: &str, root: Root) -> Option<()> {
        let mut lock = self.roots.lock().ok()?;
        lock.insert(root_name.to_string(), root);
        Some(())
    }
}
