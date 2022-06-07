use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use ahash::AHashMap;

use crate::{ResourceError, Layout};

// A resource set simply contains multiple unique resources
pub struct ResourceSet(AHashMap<TypeId, Box<dyn Resource>>);

impl ResourceSet {
    // Get a mutable reference to the boxed resource from the set by casting it first
    pub(crate) fn get_casted<T: Resource>(&mut self) -> Result<&mut T, ResourceError> {
        let boxed = self.0
            .get_mut(&TypeId::of::<T>())
            .ok_or(ResourceError::missing::<T>())?;
        Ok(boxed.as_any_mut().downcast_mut::<T>().unwrap())
    }

    // Insert a new resource into the set
    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.0.insert(TypeId::of::<R>(), Box::new(resource));
    }

    // Remove a resouce from the set
    pub fn remove<R: Resource>(&mut self) {
        self.0.remove(&TypeId::of::<R>());
    }

    // Fetch a tuple of certain resource handles from the set
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        L::validate().map(|_| unsafe { L::fetch_unchecked(self) })?
    }
}

// A resource is some shared data that will be accessed by multiple systems
pub trait Resource: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
