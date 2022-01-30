use super::OctreeNode;

// Some heuristic settings that can be applied to a simple octree to change when certain nodes subdivide
pub struct HeuristicSettings {
    // A function to check against each node
    pub(crate) function: fn(&OctreeNode, &veclib::Vector3<f32>) -> bool, 
}

impl Default for HeuristicSettings {
    fn default() -> Self {
        Self { function: |node, target| {
            // AABB intersection, return true if point in on the min edge though
            let aabb = (node.get_aabb().min.elem_lte(target) & node.get_aabb().max.elem_gt(target)).all();
            aabb
        } }
    }
}

impl HeuristicSettings {
    // Create some new heuristic settings based on the subdivide function
    pub fn new(function: fn(&OctreeNode, &veclib::Vector3<f32>) -> bool) -> Self {
        Self {
            function,
        }
    }
}
