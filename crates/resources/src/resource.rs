use std::any::{Any, TypeId};


// A single resource that can be shared within the world
pub trait Resource: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


// A resource refernce, like a & reference or &mut reference
// TODO: Rename
pub trait ResRef<'a> {
    type Inner: Resource + 'static;
    
    // Get the TypeId of the underlying raw resource
    fn id() -> TypeId {
        TypeId::of::<Self::Inner>()
    }
    
    // Convert a mutable boxed reference to the resource reference
    fn as_mut(boxed: &'a mut Box<dyn Resource>) -> Self;
}


// Auto res-ref for resources
impl<'a, T: Resource> ResRef<'a> for &'a T {
    type Inner = T;

    fn as_mut(boxed: &'a mut Box<dyn Resource>) -> Self {
        boxed.as_any().downcast_ref().unwrap()
    }
}
impl<'a, T: Resource> ResRef<'a> for &'a mut T {
    type Inner = T;
    
    fn as_mut(boxed: &'a mut Box<dyn Resource>) -> Self {
        boxed.as_any_mut().downcast_mut().unwrap()
    }
}
