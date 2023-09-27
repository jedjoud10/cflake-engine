use super::System;
use crate::events::Event;
use std::marker::PhantomData;

/// How this system is going to execute in relation to other systems
/// This allows us to set dependencies, dependants, or inject systems within both
/// using "rules" that define what must execute before a system and after a system
pub struct InjectionOrder<E: Event> {
    _phantom: PhantomData<E>,
}

impl<E: Event> Default for InjectionOrder<E> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<E: Event> InjectionOrder<E> {
    /// Make this system execute before the "other" system
    pub fn before<O: System<E>>(self) -> Self {
        self
    }

    /// Make this system execute after the "other" system
    pub fn after<O: System<E>>(self) -> Self {
        self
    }
}
