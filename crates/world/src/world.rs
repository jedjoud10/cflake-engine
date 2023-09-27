use crate::resource::Resource;
use ahash::AHashMap;
use atomic_refcell::AtomicRefCell;
use std::any::TypeId;

/// A world is a container for resources that are stored persistently throughout the game lifetime
/// Most systems will reference the internal resources directly to make use of parallelism,
/// but if you wish to "dynamically" access resources you can access the world and use the
/// ``get`` and ``get_mut`` functions to fetch resources directly.
pub struct World(pub(crate) AHashMap<TypeId, AtomicRefCell<Box<dyn Resource>>>);

impl World {}
