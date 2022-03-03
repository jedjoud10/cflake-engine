use ahash::AHashMap;
use bitfield::Bitfield;
use parking_lot::RwLock;
use slotmap::SlotMap;
use std::{any::Any, cell::UnsafeCell, sync::Arc};

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

// Component groups that we must remove
pub(crate) struct ComponentGroupToRemove {
    pub components: AHashMap<Bitfield<u32>, ComponentKey>,
    pub counter: usize,
    pub key: EntityKey,
}

// Component ref guards. This can be used to detect whenever we mutate a component
pub struct ComponentReadGuard<'a, T>
where
    T: Component,
{
    borrow: &'a T,
}

impl<'a, T> ComponentReadGuard<'a, T>
where
    T: Component,
{
    pub fn new(borrow: &'a T) -> Self {
        Self { borrow }
    }
}

impl<'a, T> std::ops::Deref for ComponentReadGuard<'a, T>
where
    T: Component,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow
    }
}
// Component mut guard
pub struct ComponentWriteGuard<'a, T>
where
    T: Component,
{
    borrow_mut: &'a mut T,
}

impl<'a, T> ComponentWriteGuard<'a, T>
where
    T: Component,
{
    pub fn new(borrow_mut: &'a mut T) -> Self {
        Self { borrow_mut }
    }
}

impl<'a, T> std::ops::Deref for ComponentWriteGuard<'a, T>
where
    T: Component,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow_mut
    }
}

impl<'a, T> std::ops::DerefMut for ComponentWriteGuard<'a, T>
where
    T: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut
    }
}
