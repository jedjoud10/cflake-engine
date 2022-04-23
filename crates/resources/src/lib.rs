// Le resource
pub use resources_derive::Resource;
pub trait Resource: anymap::any::Any {}
use anymap::AnyMap;

pub struct ResourcesSet {
    anymap: AnyMap,
}

impl Default for ResourcesSet {
    fn default() -> Self {
        Self { anymap: AnyMap::new() }
    }
}

impl ResourcesSet {
    // Insert a resource into the set
    pub fn insert<U: Resource>(&mut self, global: U) -> Option<()> {
        self.anymap.insert(global).map(|_x| ()).xor(Some(()))
    }
    // Get a resource from the set (immutably)
    pub fn get<U: Resource>(&self) -> Option<&U> {
        self.anymap.get::<U>()
    }
    // Get a resource from the set (mutably)
    pub fn get_mut<U: Resource>(&mut self) -> Option<&mut U> {
        self.anymap.get_mut::<U>()
    }
    // Remove a resource from the set
    pub fn remove<U: Resource>(&mut self) -> Option<U> {
        self.anymap.remove::<U>()
    }
}
