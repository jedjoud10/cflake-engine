use super::System;
use crate::{events::Event, world::World};
use std::{marker::PhantomData, any::TypeId};

/// How this system is going to execute in relation to other systems
/// This allows us to set dependencies, dependants, or inject systems within both
/// using "rules" that define what must execute before a system and after a system
pub struct InjectionOrder<'a, E: Event> {
    pub(crate) rules: &'a mut Vec<InjectionRule>,
    pub(crate) default: bool,
    pub(crate) _phantom: PhantomData<E>,
}

/// A rule that depicts the location of the systems relative to other systems
#[derive(Clone, Debug)]
pub enum InjectionRule {
    /// This forces the current system to execute before the other system
    Before(TypeId),

    /// This forces the current system to execute after the other system 
    After(TypeId),
}

impl<'a, E: Event> InjectionOrder<'a, E> {
    /// Make this system execute before the "other" system
    pub fn before<S: System<E>>(mut self, system: S) -> Self {
        if std::mem::take(&mut self.default) {
            self.rules.clear();
        }

        self.rules.push(InjectionRule::Before(TypeId::of::<S>()));
        self
    }

    /// Make this system execute after the "other" system
    pub fn after<S: System<E>>(mut self, system: S) -> Self {
        if std::mem::take(&mut self.default) {
            self.rules.clear();
        }

        self.rules.push(InjectionRule::After(TypeId::of::<S>()));
        self
    }
}

/// Default pre-user system
pub fn pre_user<E: Event>(_: &mut World, _: &E) {}

/// Default post-user system
pub fn post_user<E: Event>(_: &mut World, _: &E) {}

// Type ID of passed value
pub(super) fn type_id_of_val<T: 'static>(_: T) -> TypeId {
    TypeId::of::<T>()
}