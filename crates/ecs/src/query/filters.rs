use std::{intrinsics::transmute, ops::BitOr};
use crate::{Mask, EntityState, ArchetypalState, Component, registry};

// Query filter that will block certain bundles from being iterated through
pub struct QueryFilter {
}

impl QueryFilter {
}