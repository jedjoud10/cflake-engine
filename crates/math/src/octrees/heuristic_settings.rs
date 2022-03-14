use super::Node;

// Some heuristic settings that can be applied to a simple octree to change when certain nodes subdivide
pub struct HeuristicSettings {
    // A function to check against each node
    pub function: fn(&Node, &vek::Vec3<f32>) -> bool,
}

impl Default for HeuristicSettings {
    fn default() -> Self {
        Self {
            function: |node, target| crate::intersection::point_aabb(target, &node.aabb()),
        }
    }
}
