use ahash::AHashMap;
use bitfield::Bitfield;
use parking_lot::RwLock;
use slotmap::SlotMap;
use std::{any::Any, cell::{UnsafeCell, RefCell}, sync::Arc, rc::Rc};

use crate::entity::EntityKey;

slotmap::new_key_type! {
    pub struct ComponentKey;
    pub(crate) struct ComponentGroupKey;
}

// A component that can be accessed through multiple worker threads
// This allows for parralel computing, but we must be careful with reading/writing to it
pub trait Component: Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Main type because I don't want to type
pub type Components = Arc<RwLock<SlotMap<ComponentKey, UnsafeCell<EnclosedComponent>>>>;
pub type EnclosedComponent = Box<dyn Component + Sync + Send>;
pub(crate) type DanglingComponentsToRemove = Rc<RefCell<SlotMap<ComponentGroupKey, ComponentGroupToRemove>>>;

// Component groups that we must remove
pub(crate) struct ComponentGroupToRemove {
    pub components: AHashMap<Bitfield<u32>, ComponentKey>,
    pub counter: usize,
    pub key: EntityKey,
}