use super::System;
use crate::events::Event;
use std::{marker::PhantomData, any::TypeId};

/// How this system is going to execute in relation to other systems
/// This allows us to set dependencies, dependants, or inject systems within both
/// using "rules" that define what must execute before a system and after a system
pub struct InjectionOrder<'a, E: Event> {
    _phantom: PhantomData<E>,
    before: &'a mut Vec<TypeId>,
    after: &'a mut Vec<TypeId>,
}

impl<'a, E: Event> InjectionOrder<'a, E> {
    /// Make this system execute before the "other" system
    pub fn before(mut self, system: impl System<E>) -> Self {
        //self.before.push(TypeId::of::<O>());
        self
    }

    /// Make this system execute after the "other" system
    pub fn after(mut self, system: impl System<E>) -> Self {
        //self.after.push(TypeId::of::<O>());
        self
    }
}
