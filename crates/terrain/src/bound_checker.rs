use math::{bounds::AABB, octrees::*};

// The bound checker
pub struct BoundChecker {}
impl BoundChecker {
    // Check if a certain node can be spawned. This does some smart AABB bound checking, W.I.P
    pub fn bound_check(node: &OctreeNode) -> bool {
        let aabb = AABB {
            min: veclib::Vector3::new(-50000.0, 0.0, -50000.0),
            max: veclib::Vector3::new(50000.0, 10.0, 50000.0),
        };
        math::Intersection::aabb_aabb(&node.get_aabb(), &aabb)
        //true
    }
}
