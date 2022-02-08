use super::LinkedComponents;
use crate::entity::EntityID;
use ahash::AHashMap;
use std::{
    ops::{Deref, DerefMut},
    sync::MutexGuard,
};

// A guard that internally stores a mutex guard
pub struct ComponentQueryGuard<'a> {
    pub(crate) inner: MutexGuard<'a, AHashMap<EntityID, LinkedComponents>>,
}

impl<'a> Deref for ComponentQueryGuard<'a> {
    type Target = AHashMap<EntityID, LinkedComponents>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<'a> DerefMut for ComponentQueryGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}