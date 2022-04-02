use crate::{ComponentStorage, Mask};

use super::states::ComponentMutationsBitfield;
use nohash_hasher::NoHashHasher;
use std::{collections::HashMap, hash::BuildHasherDefault, sync::Arc};

// NoHash hasher that works with Mask
pub type MaskHasher = BuildHasherDefault<NoHashHasher<Mask>>;

// Unique component storages that will be cloned whenever we make a new archetype (cheap since the vectors are empty)
pub(crate) type UniqueComponentStoragesHashMap = HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher>;
