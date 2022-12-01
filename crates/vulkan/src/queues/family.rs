use crate::{Pool, Queue};
use smallvec::SmallVec;

// Virtual queue family to dissacociate from actual queue family
// Virtual simply means that it creates two family structs for the same queue family index
pub struct Family {
    queues_count_init: usize,
    pools: Vec<Pool>,
    queues: SmallVec<[Queue; 1]>,
}
