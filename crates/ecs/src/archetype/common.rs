use nohash_hasher::{IsEnabled, NoHashHasher};
use std::{collections::HashMap, hash::BuildHasherDefault};

use crate::component::SparseComponentStates;

use super::VecComponentStorage;

// A component storage that might not be initialized
pub type MaybeNoneStorage = Option<Box<dyn VecComponentStorage>>;

// The components hashmap
pub type NoHash = BuildHasherDefault<NoHashHasher<u64>>;
pub type ComponentsHashMap = HashMap<u64, (MaybeNoneStorage, SparseComponentStates), NoHash>;

// Identifier
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ArchetypeId(pub(crate) u64);
impl std::hash::Hash for ArchetypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        u64::hash(&self.0, state)
    }
}
impl IsEnabled for ArchetypeId {}
