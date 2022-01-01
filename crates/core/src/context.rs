use std::{cell::{RefCell, Ref, RefMut}, sync::Arc};

use no_deadlocks::RwLock;

// An owned context that we can create inside the lazy_static! macro
pub struct OwnedContext<T> {
    val: Arc<RefCell<T>>
}

impl<T> Default for OwnedContext<T> where T: Default {
    fn default() -> Self {
        Self { val: Arc::new(RefCell::new(T::default())) }
    }
}

impl<T> OwnedContext<T> {
    // Create a new OwnedContext from a default value
    pub fn new(val: T) -> Self {
        Self {
            val: Arc::new(RefCell::new(val))
        }
    }
    // Update the owned value using a closure
    pub fn update<F>(&self, f: F)
        where F: FnOnce(&mut T)
    {
        let val = self.val.as_ref().borrow_mut();
        let val = &mut *val;
        f(val);
    }
    // Get the value
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.val.as_ref().borrow()
    }
    // Get the value mutably
    pub fn borrow_mut<'a>(&'a mut self) -> RefMut<'a, T> {
        self.val.as_ref().borrow_mut()
    }
}

// Some context. This can be used to share data about the world
pub struct RefContext<T> {
    val: Arc<RefCell<T>>
}