use std::num::{NonZeroU64};

use math::octrees::Node;

// The data that will be used to store the position/scale of the chunk
#[derive(Clone, Copy)]
pub struct ChunkCoords {
    pub position: vek::Vec3<i64>,
    pub size: NonZeroU64,
    pub depth: u8,
}

// Generate the chunk coords from an octree node
impl ChunkCoords {
    // New from chunk coords
    pub fn new(octree_node: &Node) -> Self {
        Self {
            position: octree_node.position(),
            size: NonZeroU64::new(octree_node.half_extent().get() * 2).unwrap(),
            depth: octree_node.depth(),
        }
    }
    // Get the center of the chunk coordinates
    pub fn center(&self) -> vek::Vec3<i64> {
        self.position + (self.size.get() as i64) / 2
    }
}

// Equality tests
impl PartialEq for ChunkCoords {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.depth == other.depth
    }
}
impl Eq for ChunkCoords {}
impl std::hash::Hash for ChunkCoords {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.depth.hash(state);
    }
}
