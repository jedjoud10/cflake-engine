use super::LinkedComponents;
use crate::entity::EntityID;
use ahash::AHashMap;
use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

pub struct MutComponentQuery<'a> {
    pub(crate) inner: RefMut<'a, AHashMap<EntityID, LinkedComponents>>,
}

impl<'a> Deref for MutComponentQuery<'a> {
    type Target = AHashMap<EntityID, LinkedComponents>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<'a> DerefMut for MutComponentQuery<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

pub struct RefComponentQuery<'a> {
    pub(crate) inner: Ref<'a, AHashMap<EntityID, LinkedComponents>>,
}

impl<'a> Deref for RefComponentQuery<'a> {
    type Target = AHashMap<EntityID, LinkedComponents>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
