use getset::{CopyGetters, Getters};
use slotmap::{Key, SlotMap};

use super::{node::Node, HeuristicSettings, NodeKey};

// A simple octree, no incremental generation what so ever
#[derive(Getters, CopyGetters)]
pub struct Octree {
    // The old target point
    target: Option<vek::Vec3<f32>>,
    // The total nodes in the octree
    #[getset(get = "pub")]
    nodes: SlotMap<NodeKey, Node>,
    #[getset(get_copy = "pub")]
    root: NodeKey,
    // The depth of the tree
    #[getset(get_copy = "pub")]
    depth: u8,
    // The size factor for each node, should be a power of two
    #[getset(get_copy = "pub")]
    size: u64,
    // Some specific heuristic settings
    #[getset(get = "pub")]
    hsettings: HeuristicSettings,
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
        let root = nodes.insert_with_key(|key| Node::root(key, depth, size));
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
    pub fn update(&mut self, target: vek::Vec3<f32>) -> Option<()> {
        // Simple check to see if we even moved lol
        if let Some(pos) = self.target.as_ref() {
            // Check distances
            if vek::Vec3::<f32>::distance(*pos, target) < (self.size / 2) as f32 {
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
            let key = pending_nodes.remove(0);
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
                *self.nodes.get_mut(key).unwrap().children_mut() = Some(children_keys);
                pending_nodes.extend(children_keys);
            }
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
            let octree_node = pending_nodes.remove(0);

            // If the node function is true, we recursively iterate through the children
            if function(octree_node) {
                if let Some(children) = &octree_node.children() {
                    // Add the children if we have them
                    for child_id in children {
                        pending_nodes.push(self.nodes.get(*child_id).unwrap())
                    }
                }
            }
        }
    }
}
