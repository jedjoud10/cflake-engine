use super::{node::OctreeNode, HeuristicSettings};
use ordered_vec::simple::UnversionnedOrderedVec;

// A simple octree, no incremental generation what so ever
#[derive(Default)]
pub struct Octree {
    // The old target point
    pub target: Option<veclib::Vector3<f32>>,
    // The total nodes in the octree
    pub nodes: UnversionnedOrderedVec<OctreeNode>,
    // The depth of the tree
    pub depth: u8,
    // The size factor for each node, should be a power of two
    pub size: u64,
    // Some specific heuristic settings
    pub hsettings: HeuristicSettings,
}

impl Octree {
    // Create a new octree with a specific depth
    pub fn new(depth: u8, size: u64, hsettings: HeuristicSettings) -> Self {
        Self {
            target: None,
            nodes: UnversionnedOrderedVec::default(),
            size,
            depth,
            hsettings,
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
    pub fn update(&mut self, target: veclib::Vector3<f32>) -> Option<()> {
        // Clear
        self.nodes.clear();
        // Simple check to see if we even moved lol
        if let Some(pos) = self.target.as_ref() {
            // Check distances
            if veclib::Vector3::<f32>::distance(*pos, target) < self.size as f32 {
                return None;
            }
        }

        let root_node = self.get_root_node();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        // The default root node
        pending_nodes.push(root_node.clone());
        self.nodes.push_shove(root_node);

        // Evaluate each node
        while !pending_nodes.is_empty() {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth, &self.hsettings) {
                // Add each child node, but also update the parent's child link id
                let nodes_to_push = octree_node.subdivide(&mut self.nodes);
                pending_nodes.extend(nodes_to_push.clone());
            }

            // Don't cause an infinite loop
            pending_nodes.remove(0);
        }

        self.target = Some(target);
        Some(())
    }
}
