use super::node::OctreeNode;

// A simple octree, no incremental generation what so ever
pub struct Octree {
    // The target node
    pub target_node: Option<OctreeNode>
    // The total nodes in the octree
    pub nodes: Vec<Option<OctreeNode>>,
    // The depth of the tree
    pub depth: u8,
}


impl Octree {
    // Generate an octree from a root and a target point
    pub fn generate_octree(&self, target: &veclib::Vector3<f32>, root_node: OctreeNode) -> (Vec<OctreeNode>, Option<OctreeNode>) {
        // The final nodes
        let mut nodes: Vec<OctreeNode> = Vec::new();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        // The default root node
        pending_nodes.push(root_node.clone());
        nodes.push(root_node.clone());

        // The targetted node that is specified using the target position
        let mut targetted_node: Option<OctreeNode> = None;
        let mut closest_dist: f32 = f32::MAX;

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // Check if this octree node is the targeted node
            let dist = veclib::Vector3::<f32>::from(octree_node.get_center()).distance(*target);
            // Check distances and depth values
            if octree_node.depth == self.depth - 1 && dist < closest_dist {
                targetted_node = Some(octree_node.clone());
                closest_dist = dist;
            }
            
            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth) {
                pending_nodes.extend(octree_node.subdivide());
            }
            
            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
            // Update the node if it was the root node
            if nodes.len() == 1 {
                // This is the root node, update it
                nodes[0] = octree_node;
            } else {
                // This is not the root node
                nodes.push(octree_node);
            }
        }
        return (nodes, targetted_node);
    }
}