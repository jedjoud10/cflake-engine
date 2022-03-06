use std::marker::PhantomData;

use crate::utils::ComponentError;

use super::{Component, ComponentKey, ComponentSet};

// Component fetcher that you could use to get components that don't exist in the current query
// This cannot be used in multiple threads
pub struct RefComponentFetcher<'a> {
    _phantom: PhantomData<*const u32>,
    set: &'a ComponentSet,
}

impl<'a> RefComponentFetcher<'a> {
    // Create a new components fetcher
    pub fn new(set: &'a ComponentSet) -> RefComponentFetcher {
        Self {
            set,
            _phantom: Default::default(),
        }
    }
    // Get a single component (should not be multithreaded!)
    pub fn get<T>(&self, key: ComponentKey) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        self.set.get(key)
    }
}

// Mutable component fetcher so we can mutate each component if we want to
pub struct MutComponentFetcher<'a> {
    _phantom: PhantomData<*const u32>,
    set: &'a mut ComponentSet,
}

impl<'a> MutComponentFetcher<'a> {
    // Create a new components fetcher
    pub fn new(set: &'a mut ComponentSet) -> MutComponentFetcher {
        Self {
            set,
            _phantom: Default::default(),
        }
    }
    // Get a single component (should not be multithreaded!)
    pub fn get<T>(&self, key: ComponentKey) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        self.set.get(key)
    }
    // Get a single component mutably (should not be multithreaded!)
    pub fn get_mut<T>(&mut self, key: ComponentKey) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        self.set.get_mut(key)
    }
}