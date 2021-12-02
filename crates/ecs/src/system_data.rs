use std::any::Any;

use ecs::*;

// Some system data
#[derive(Default)]
pub struct SystemData {
    // Internal system data
    pub internal_system_data: Option<Box<dyn InternalSystemData>>,
}

impl SystemData {
    // Turn a specific struct into a system data
    pub fn convert<T: InternalSystemData + 'static>(&mut self, system_data: T) {
        self.internal_system_data = Some(Box::new(system_data));
    }
    // Cast the internal system data to a custom system data generic type
    pub fn cast<T: InternalSystemData + 'static>(&self) -> Option<&T> {
        let any = self.internal_system_data.as_ref()?.as_any();
        let output = any.downcast_ref::<T>()?;
        return Some(output);
    }
    // Do the same as above but with a mutable reference this time
    pub fn cast_mut<T: InternalSystemData + 'static>(&mut self) -> Option<&mut T> {
        let any = self.internal_system_data.as_mut()?.as_any_mut();
        let output = any.downcast_mut::<T>()?;
        return Some(output);
    }
    // As any
}

// An internal system data trait. The system data is stored on the heap with a Box pointer
pub trait InternalSystemData {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
