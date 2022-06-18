use crate::{Layout, ResourceError, StorageSet};
use ahash::AHashMap;
use std::any::{Any, TypeId};

// A resource set simply contains multiple unique resources
#[derive(Default)]
pub struct ResourceSet(AHashMap<TypeId, Box<dyn Resource>>);

impl ResourceSet {
    // Get a mutable reference to the boxed resource from the set by casting it first
    pub(crate) fn get_casted<T: Resource>(&mut self) -> Result<&mut T, ResourceError> {
        let boxed = self
            .0
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

    // Remove a resouce from the set (if possible)
    // This returns true if we successfully deleted the resource
    pub fn remove<R: Resource>(&mut self) -> bool {
        if R::can_remove() {
            self.0.remove(&TypeId::of::<R>()).is_some()
        } else {
            false
        }
    }

    // Fetch a tuple of certain resource handles from the set
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        L::validate().map(|_| unsafe { L::fetch_unchecked(self) })?
    }

    // Check if a resource is contained within the set
    pub fn contains<R: Resource>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<R>())
    }

    // Get a set of all the inner storage resources
    pub fn storages<'a>(&'a mut self) -> StorageSet<'a> {
        StorageSet(self)
    }

    // Method that is called before any systems are executed
    pub fn start_frame(&mut self) {
        for (_, resource) in self.0.iter_mut() {
            resource.start_frame()
        }
    }

    // Method that is called after all the systems have executed
    // TODO: Hide this from the external API
    pub fn end_frame(&mut self) {
        for (_, resource) in self.0.iter_mut() {
            resource.end_frame()
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

    // A function that will be called right before the resource gets fetch
    fn pre_fetch(_set: &mut ResourceSet)
    where
        Self: Sized + 'static,
    {
    }

    // A function that enables/disables removal for this specific resource
    fn can_remove() -> bool
    where
        Self: Sized,
    {
        true
    }

    // And an update function that runs every frame the resource is stored in the set
    fn start_frame(&mut self) {}

    // Update function that runs after all the systems execute
    fn end_frame(&mut self) {}
}
