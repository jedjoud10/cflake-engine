use bitfield::Bitfield;
use std::any::Any;

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

// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
// A component trait that can be added to other components
pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_component_name() -> String
    where
        Self: Sized;
}

// Main type because I don't want to type
pub type EnclosedComponent = Box<dyn Component + Sync + Send>;
pub type EnclosedGlobalComponent = Box<dyn Component + Sync + Send>;

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
