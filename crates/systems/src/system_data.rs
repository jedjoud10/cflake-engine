use std::any::Any;

use ecs::*;
use errors::ECSError;

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
    // As any
    pub fn as_any(&self) -> &dyn Any {
        self
    }
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// An internal system data trait. The system data is stored on the heap with a Box pointer
pub trait InternalSystemData {
}