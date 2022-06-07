use std::{
    any::{Any, TypeId},
    cell::RefCell,
};
use ahash::AHashMap;
use crate::{ResourceError, Layout};

// A resource set simply contains multiple unique resources
#[derive(Default)]
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
        let mut boxed = Box::new(resource);
        boxed.added();
        self.0.insert(TypeId::of::<R>(), boxed);
    }

    // Remove a resouce from the set
    pub fn remove<R: Resource>(&mut self) {
        self.0.remove(&TypeId::of::<R>());
    }

    // Fetch a tuple of certain resource handles from the set
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        L::validate().map(|_| unsafe { L::fetch_unchecked(self) })?
    }

    // This function should be called everyframe so it can update the internal resources
    pub fn update(&mut self) {
        for (_, resource) in self.0.iter_mut() {
            resource.update()
        }
    }
}

// A resource is some shared data that will be accessed by multiple systems
pub trait Resource: 'static {
    // Bruh conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Some resources can have a init function that gets ran when the resource gets added onto the set
    fn added(&mut self) {}

    // And an update function that runs every frame the resource is stored in the set
    fn update(&mut self) {}
}
