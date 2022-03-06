use slotmap::{Key, SlotMap};

use super::{node::Node, HeuristicSettings, NodeKey};

// A simple octree, no incremental generation what so ever
pub struct Octree {
    // The old target point
    pub target: Option<veclib::Vector3<f32>>,
    // The total nodes in the octree
    pub nodes: SlotMap<NodeKey, Node>,
    pub root: NodeKey,
    // The depth of the tree
    pub depth: u8,
    // The size factor for each node, should be a power of two
    pub size: u64,
    // Some specific heuristic settings
    pub hsettings: HeuristicSettings,
}

impl Default for Octree {
    fn default() -> Self {
        Self::new(4, 32, HeuristicSettings::default())
    }
}

impl Octree {
    // Create a new octree with a specific depth
    pub fn new(depth: u8, size: u64, hsettings: HeuristicSettings) -> Self {
        // Create the root node
        let mut nodes = SlotMap::<NodeKey, Node>::default();
        let root = nodes.insert_with_key(|key| {
            // Get the maximum size of the root node
            let root_size = (2_u64.pow(depth as u32) * size as u64) as i64;
            let root_position = veclib::Vector3::<i64>::new(-(root_size / 2), -(root_size / 2), -(root_size / 2));
            // Output the root node
            Node {
                position: root_position,
                half_extent: (root_size / 2) as u64,
                depth: 0,
                parent_key: NodeKey::null(),
                key,
                children_keys: None,
            }
        });
        Self {
            target: None,
            nodes,
            root,
            size,
            depth,
            hsettings,
        }
    }

    // Get the root node of this octree
    pub fn get_root_node(&self) -> &Node {
        self.nodes.get(self.root).unwrap()
    }
    // Generate an octree from a root and a target point
    pub fn update(&mut self, target: veclib::Vector3<f32>) -> Option<()> {
        // Simple check to see if we even moved lol
        if let Some(pos) = self.target.as_ref() {
            // Check distances
            if veclib::Vector3::<f32>::distance(*pos, target) < self.hsettings.min_threshold_distance as f32 {
                return None;
            }
        }
        // Reset the tree
        self.nodes.retain(|key, _| key == self.root);
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<NodeKey> = vec![self.root];
        // Evaluate each node
        while !pending_nodes.is_empty() {
            // Get the current pending node
            let key = pending_nodes[0].clone();
            let octree_node = self.nodes.get_mut(key).unwrap();

            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth, &self.hsettings) {
                // Subidivide
                let subdivided = octree_node.subdivide();
                drop(octree_node);

                // Insert the new nodes into the tree
                let mut children_keys = [NodeKey::null(); 8];
                for (i, node) in subdivided.into_iter().enumerate() {
                    children_keys[i] = self.nodes.insert(node);
                }
                self.nodes.get_mut(key).unwrap().children_keys = Some(children_keys.clone());
                pending_nodes.extend(children_keys);
            }

            // Don't cause an infinite loop
            pending_nodes.remove(0);
        }

        self.target = Some(target);
        Some(())
    }
    // Recursively iterate through each node, and check it's children if the given function returns true
    pub fn recurse<'a>(&'a self, mut function: impl FnMut(&'a Node) -> bool) {
        // The nodes that must be evaluated
        let root_node = self.get_root_node();
        let mut pending_nodes: Vec<&'a Node> = vec![root_node];
        // Evaluate each node
        while !pending_nodes.is_empty() {
            // Get the current pending node
            let octree_node = pending_nodes[0];

            // If the node function is true, we recursively iterate through the children
            if function(octree_node) {
                if let Some(children) = &octree_node.children_keys {
                    // Add the children if we have them
                    for child_id in children {
                        pending_nodes.push(self.nodes.get(*child_id).unwrap())
                    }
                }
            }

            // Don't cause an infinite loop
            pending_nodes.remove(0);
        }
    }
}
