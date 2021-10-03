use math::{bounds::AABB, octree::OctreeNode};

// The bound checker
pub struct BoundChecker {}
impl BoundChecker {
    // Check if a certain node can be spawned. This does some smart AABB bound checking, W.I.P
    pub fn bound_check(node: &OctreeNode) -> bool {
        true
    }
}
