use math::{bounds::AABB, octree::OctreeNode};

// The bound checker
pub struct BoundChecker {
}
impl BoundChecker {
    // Check if a certain node can be spawned. This does some smart AABB bound checking, W.I.P
    pub fn bound_check(node: &OctreeNode) -> bool {
        let aabb = AABB { min: veclib::Vector3::new(-50000.0, -60.0, -50000.0), max: veclib::Vector3::new(50000.0, 30.0, 50000.0) };
        math::Intersection::aabb_aabb(&node.get_aabb(), &aabb);
        true
    }
}