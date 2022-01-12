use std::any::Any;

use crate::{ComponentError, ComponentID};
// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
// A component trait that can be added to other components
pub trait Component: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_component_name() -> String
    where
        Self: Sized;
}

// Cast a boxed component to a reference of that component
pub(crate) fn cast_component<'a, T>(linked_component: &'a dyn Component) -> Result<&T, ComponentError>
where
    T: Component + Send + Sync + 'static,
{
    let component_any: &dyn Any = linked_component.as_any();
    let reference = component_any
        .downcast_ref::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference)
}
// Cast a boxed component to a mutable reference of that component
pub(crate) fn cast_component_mut<'a, T>(linked_component: &'a mut dyn Component) -> Result<&mut T, ComponentError>
where
    T: Component + Send + Sync + 'static,
{
    let component_any: &mut dyn Any = linked_component.as_any_mut();
    let reference_mut = component_any
        .downcast_mut::<T>()
        .ok_or_else(|| ComponentError::new_without_id("Could not cast component".to_string()))?;
    Ok(reference_mut)
}

// Main type because I don't want to type
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
