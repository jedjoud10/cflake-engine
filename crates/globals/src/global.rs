use std::any::Any;
// A global component that can only be accessed by the main thread
// This means that it doesn't need to be Sync
pub trait Global {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
pub type EnclosedGlobalComponent = Box<dyn Global>;

// Global ref guards. This can be used to detect whenever we mutate a global
pub struct GlobalReadGuard<'a, T>
where
    T: Global,
{
    borrow: &'a T,
}

impl<'a, T> GlobalReadGuard<'a, T>
where
    T: Global,
{
    pub fn new(borrow: &'a T) -> Self {
        Self { borrow }
    }
}

impl<'a, T> std::ops::Deref for GlobalReadGuard<'a, T>
where
    T: Global,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow
    }
}
// Component mut guard
pub struct GlobalWriteGuard<'a, T>
where
    T: Global,
{
    borrow_mut: &'a mut T,
}

impl<'a, T> GlobalWriteGuard<'a, T>
where
    T: Global,
{
    pub fn new(borrow_mut: &'a mut T) -> Self {
        Self { borrow_mut }
    }
}

impl<'a, T> std::ops::Deref for GlobalWriteGuard<'a, T>
where
    T: Global,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow_mut
    }
}

impl<'a, T> std::ops::DerefMut for GlobalWriteGuard<'a, T>
where
    T: Global,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut
    }
}
