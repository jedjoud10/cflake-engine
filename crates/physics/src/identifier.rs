use std::marker::PhantomData;

use enum_as_inner::EnumAsInner;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

// ID for a specific physics related object
#[derive(Clone, Copy)]
pub struct PhysicsID<T> {
    pub(crate) inner: PhysicsIDType,
    _phantom: PhantomData<T>,
}

impl<T> PhysicsID<T> {
    // Le new
    pub(crate) fn new(inner: PhysicsIDType) -> Self {
        Self {
            inner,
            _phantom: PhantomData::default(),
        }
    }
}

// Internal
#[derive(EnumAsInner)]
#[derive(Clone, Copy)]
pub(crate) enum PhysicsIDType {
    RigidBody(RigidBodyHandle),
    Collider(ColliderHandle),
    Surface(usize),
}
