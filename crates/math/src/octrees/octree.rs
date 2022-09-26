use super::{node::Node, NodeKey};
use crate::AABB;
use slotmap::{Key, SlotMap};

type BoxedHeuriticFunc = Box<dyn Fn(&Node, &vek::Vec3<f32>) -> bool>;

// The octree heuristic is what we must use within the octree to specify what nodes will be further subdivided
pub enum OctreeHeuristic {
    // THe heuristic function is a boxed function
    ManualBoxed(BoxedHeuriticFunc),

    // The heuristic function is a static global function
    Manual(fn(&Node, &vek::Vec3<f32>) -> bool),

    // The heuristic function uses some sort of LOD logic
    LodHeuristic {
        min_radius_lod: f32,
        falloff: f32,
        exp_falloff: f32,
    },

    // The heuristic function uses a sphere as a bound area for intersections
    SphereHeuristic {
        radius: f32,
    },

    // The heursistic function uses an AABB as a bound area for intersections
    AABBHeuristic {
        extent: f32,
    },
}

// A simple octree, no incremental generation what so ever
pub struct Octree {
    // The old target point
    target: Option<vek::Vec3<f32>>,

    // The total nodes in the octree
    nodes: SlotMap<NodeKey, Node>,
    root: NodeKey,

    // The depth of the tree
    depth: u8,

    // The size factor for each node, must be a power of two
    size: u64,

    // Some specific heuristic settings
    heuristic: BoxedHeuriticFunc,
}

impl Octree {
    // Create a new octree with a specific depth and pass function
    pub fn new(depth: u8, size: u64, heuristic: OctreeHeuristic) -> Self {
        let mut nodes = SlotMap::<NodeKey, Node>::default();
        let root = nodes.insert_with_key(|key| Node::root(key, depth, size));

        let heuristic = match heuristic {
            OctreeHeuristic::ManualBoxed(b) => b,
            OctreeHeuristic::Manual(x) => Box::new(x),
            OctreeHeuristic::LodHeuristic {
                min_radius_lod: _,
                falloff: _,
                exp_falloff: _,
            } => Box::new(|_node: &Node, _loc: &vek::Vec3<f32>| false),
            OctreeHeuristic::SphereHeuristic { radius: _ } => {
                Box::new(|_node: &Node, _loc: &vek::Vec3<f32>| false)
            }
            OctreeHeuristic::AABBHeuristic { extent } => {
                Box::new(move |node: &Node, loc: &vek::Vec3<f32>| {
                    let user = AABB {
                        min: loc - vek::Vec3::broadcast(extent / 2.0),
                        max: loc + vek::Vec3::broadcast(extent / 2.0),
                    };

                    let node = node.aabb();

                    crate::aabb_aabb(&user, &node)
                })
            }
        };

        Self {
            target: None,
            nodes,
            root,
            size,
            depth,
            heuristic,
        }
    }

    // Get the nodes immutably
    pub fn nodes(&self) -> &SlotMap<NodeKey, Node> {
        &self.nodes
    }

    // Get the nodes mutably
    pub fn nodes_mut(&mut self) -> &mut SlotMap<NodeKey, Node> {
        &mut self.nodes
    }

    // Get the depth of the octree
    pub fn depth(&self) -> u8 {
        self.depth
    }

    // Get the size of the octree
    pub fn size(&self) -> u64 {
        self.size
    }

    // Get the root node immutably
    pub fn root_node(&self) -> &Node {
        self.nodes.get(self.root).unwrap()
    }

    // Get the root node mutably
    pub fn root_mode_mut(&mut self) -> &mut Node {
        self.nodes.get_mut(self.root).unwrap()
    }

    // Check if we must update the octree
    pub fn must_update(&self, target: vek::Vec3<f32>) -> bool {
        // Simple check to see if we even moved lol
        if let Some(pos) = self.target.as_ref() {
            // Check distances
            vek::Vec3::<f32>::distance(*pos, target) > (self.size / 2) as f32
        } else {
            true
        }
    }

    // Update the internal octree using the target point
    // This will immediately return false if we cannot update the octree
    pub fn update(&mut self, target: vek::Vec3<f32>) -> bool {
        if !self.must_update(target) {
            return false;
        }

        // Evaluate each node
        self.nodes.retain(|key, _| key == self.root);
        let mut pending_nodes: Vec<NodeKey> = vec![self.root];

        while !pending_nodes.is_empty() {
            // Get the current pending node
            let key = pending_nodes.remove(0);
            let node = self.nodes.get_mut(key).unwrap();

            // Check if we can subdivide the node
            if (self.heuristic)(node, &target) && node.depth() < self.depth - 1 {
                let subdivided = node.subdivide();

                // Insert the new nodes into the tree
                let mut keys = [NodeKey::null(); 8];
                for (i, node) in subdivided.into_iter().enumerate() {
                    keys[i] = self.nodes.insert(node);
                }

                *self.nodes.get_mut(key).unwrap().children_mut() = Some(keys);
                pending_nodes.extend(keys);
            }
        }

        self.target = Some(target);
        true
    }

    // Recursively iterate through each node, and check it's children if the given function returns true
    // This will return a vector containing all the nodes that have passed the test
    pub fn recurse<'a>(&'a self, mut function: impl FnMut(&'a Node) -> bool) -> Vec<&'a Node> {
        let mut pending_nodes: Vec<&'a Node> = vec![self.root_node()];
        let mut passed: Vec<&'a Node> = Vec::new();

        while !pending_nodes.is_empty() {
            let node = pending_nodes.remove(0);

            // If the node function is true, we recursively iterate through it's children
            if function(node) {
                passed.push(node);
                if let Some(children) = node.children() {
                    for key in children {
                        pending_nodes.push(self.nodes.get(*key).unwrap())
                    }
                }
            }
        }

        passed
    }
}
