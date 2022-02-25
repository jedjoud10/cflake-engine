use super::Node;

// Some heuristic settings that can be applied to a simple octree to change when certain nodes subdivide
pub struct HeuristicSettings {
    // A function to check against each node
    pub(crate) function: fn(&Node, &veclib::Vector3<f32>) -> bool,
    // The minimum distance the target needs to move before we recompute the octree
    pub(crate) min_threshold_distance: f32,
}

impl Default for HeuristicSettings {
    fn default() -> Self {
        Self {
            function: |node, target| {
                // AABB intersection, return true if point in on the min edge though

                (node.get_aabb().min.elem_lte(target) & node.get_aabb().max.elem_gt(target)).all()
            },
            min_threshold_distance: 16.0,
        }
    }
}

impl HeuristicSettings {
    // Create some new heuristic settings based on the subdivide function
    pub fn with_function(
        mut self,
        function: fn(&Node, &veclib::Vector3<f32>) -> bool,
    ) -> Self {
        self.function = function;
        self
    }
    // Modify the threshold
    pub fn with_threshold(mut self, min_threshold_distance: f32) -> Self {
        self.min_threshold_distance = min_threshold_distance;
        self
    }
}
