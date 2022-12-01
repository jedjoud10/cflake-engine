use crate::{Adapter, Device, Instance};
use ash::vk;
use parking_lot::Mutex;

use utils::{BitSet, ImmutableVec, ThreadPool};

// Queues families and their queues that will be used by the logical device
pub struct Queues {
}