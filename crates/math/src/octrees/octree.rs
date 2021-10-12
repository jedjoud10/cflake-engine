use others::SmartList;

use super::node::{self, OctreeNode};

// A simple octree, no incremental generation what so ever
#[derive(Default)]
pub struct Octree {
    // The target node
    pub target_node: Option<OctreeNode>,
    // The total nodes in the octree
    pub nodes: SmartList<OctreeNode>,
    // The depth of the tree
    pub depth: u8,
    // The size factor for each node, should be a power of two
    pub size: u64,
}

impl Octree {
    // Create a new octree with a specific depth
    pub fn create_octree(depth: u8, size: u64) -> Self {
        Self {
            target_node: None,
            nodes: SmartList::default(),
            size,
            depth,
        }
    }
    // Get the root node of this octree
    pub fn get_root_node(&self) -> OctreeNode {
        // Get the maximum size of the root node
        let root_size = (2_u64.pow(self.depth as u32) * self.size as u64) as i64;
        let root_position = veclib::Vector3::<i64>::new(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        // Output the root node
        OctreeNode {
            position: root_position,
            half_extent: (root_size / 2) as u64,
            depth: 0,
            parent_index: 0,
            index: 0,
            children_indices: None,
        }
    }
    // Generate an octree from a root and a target point
    pub fn generate_octree(&mut self, target: &veclib::Vector3<f32>, root_node: OctreeNode) {
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        // The default root node
        pending_nodes.push(root_node.clone());
        self.nodes.add_element(root_node);

        // The targetted node that is specified using the target position
        let mut targetted_node: Option<OctreeNode> = None;

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // Update target node
            if octree_node.depth == self.depth - 1 && octree_node.can_subdivide(target, self.depth + 1) {
                targetted_node = Some(octree_node.clone());
            }

            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth) {
                // Add each child node, but also update the parent's child link id
                let nodes_to_push = octree_node.subdivide(&mut self.nodes);
                pending_nodes.extend(nodes_to_push.clone());
            }

            // Don't cause an infinite loop
            pending_nodes.remove(0);
        }

        self.target_node = targetted_node;
    }
    // Externally update the octree with nodes and a target node
    pub fn extern_update(&mut self, target_node: Option<OctreeNode>, nodes: SmartList<OctreeNode>) {
        self.target_node = target_node;
        self.nodes = nodes;
    }
}
