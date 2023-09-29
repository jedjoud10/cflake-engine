use crate::resource::Resource;
use ahash::AHashMap;
use atomic_refcell::AtomicRefCell;
use std::any::TypeId;

/// A world is a container for resources that are stored persistently throughout the game lifetime
pub struct World(pub(crate) AHashMap<TypeId, AtomicRefCell<Box<dyn Resource>>>);

/// A WorldView allows you to access immutable/mutable resources from the world in parallel with other systems
/// You can access resources that you are allowed to access given by your systems' "access" mask
/// If you try accessing a resource that you are not allowed to, the system will panic
pub struct WorldView<'a> {
    immutable: AHashMap<TypeId, &'a dyn Resource>,
    mutable: AHashMap<TypeId, &'a mut dyn Resource>,
}

impl World {}
