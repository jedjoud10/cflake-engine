use std::cell::{RefMut, Ref};

use crate::Resource;

// A read guard is an immutable reference to a resource
pub struct Read<'a, R: Resource>(pub(crate) Ref<'a, R>);

// A write guard is a mutable reference to a resource
pub struct Write<'a, R: Resource>(pub(crate) RefMut<'a, R>);