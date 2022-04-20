// Le global
pub use globals_derive::Global;
pub trait Global: anymap::any::Any {}
use anymap::AnyMap;

pub struct GlobalsSet {
    anymap: AnyMap,
}

impl Default for GlobalsSet {
    fn default() -> Self {
        Self { anymap: AnyMap::new() }
    }
}

impl GlobalsSet {
    // Insert a global into the set
    pub fn insert<U: Global>(&mut self, global: U) -> Option<()> {
        self.anymap.insert(global).map(|x| ()).xor(Some(()))
    }
    // Get a global from the set (immutably)
    pub fn get<U: Global>(&self) -> Option<&U> {
        self.anymap.get::<U>()
    }
    // Get a global from the set (mutably)
    pub fn get_mut<U: Global>(&mut self) -> Option<&mut U> {
        self.anymap.get_mut::<U>()
    }
    // Remove a global from the set
    pub fn remove<U: Global>(&mut self) -> Option<U> {
        self.anymap.remove::<U>()
    }
}
