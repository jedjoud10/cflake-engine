use crate::{component::ComponentStatesBitfield, prelude::Mask};

use super::ComponentStorage;
use nohash_hasher::{IsEnabled, NoHashHasher};
use std::{collections::HashMap, hash::BuildHasherDefault};

// NoHash hasher that works with Mask
pub type MaskHasher = BuildHasherDefault<NoHashHasher<Mask>>;

// A tuple that contains a component storage and it's corresponding mutated bitfield
type Combined = (Box<dyn ComponentStorage>, ComponentStatesBitfield);

// Component storages hash map that contains each component vector
pub(crate) type ComponentStoragesHashMap = HashMap<Mask, Combined, MaskHasher>;

// Unique component storages that will be cloned whenever we make a new archetype (cheap since the vectors are empty)
pub(crate) type UniqueComponentStoragesHashMap =
    HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher>;
