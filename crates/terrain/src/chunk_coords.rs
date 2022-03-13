use math::octrees::Node;

// The data that will be used to store the position/scale of the chunk
#[derive(Default, Clone, Copy, Debug)]
pub struct ChunkCoords {
    pub position: vek::Vec3<i64>,
    pub center: vek::Vec3<i64>,
    pub size: u64,
    pub depth: u8,
}

// Generate the chunk coords from an octree node
impl ChunkCoords {
    // New from chunk coords
    pub fn new(octree_node: &Node) -> Self {
        Self {
            position: octree_node.position(),
            center: octree_node.center(),
            size: octree_node.half_extent() * 2,
            depth: octree_node.depth(),
        }
    }
}

// Equality tests
impl PartialEq for ChunkCoords {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.depth == other.depth
    }
}
impl Eq for ChunkCoords {}
impl std::hash::Hash for ChunkCoords {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.center.hash(state);
        // We will also hash the depth for good measure
        self.depth.hash(state);
    }
}
