use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use parking_lot::RwLock;
use std::{any::Any, cell::UnsafeCell, sync::Arc};

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub(crate) cbitfield: Bitfield<u32>,
    pub(crate) idx: u64,
}
impl ComponentID {
    // Create a new component ID
    pub(crate) fn new(cbitfield: Bitfield<u32>, id: u64) -> Self {
        Self { cbitfield, idx: id }
    }
}

// A component that can be accessed through multiple worker threads
// This allows for parralel computing, but we must be careful with reading/writing to it
pub trait Component: Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Main type because I don't want to type
pub type ComponentsCollection = Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>;
pub type EnclosedComponent = Box<dyn Component + Sync + Send>;

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
