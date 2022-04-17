use crate::{registry, Component, ComponentStateRow, Entity, LayoutAccess, Mask};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::{
    cell::Cell,
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

// This will be given to the filters
#[derive(Clone, Copy)]
pub struct Input<'a>(pub(crate) &'a ComponentStateRow);

// Le filter
pub type FilterFunc = fn(Input) -> bool;

// Only components that were added will pass this filter
pub fn added<T: Component>(i: Input) -> bool {
    let mask = registry::mask::<T>();
    i.0.added(mask.offset())
}
// Only components that were mutated will pass this filter
pub fn mutated<T: Component>(i: Input) -> bool {
    let mask = registry::mask::<T>();
    i.0.mutated(mask.offset())
}
