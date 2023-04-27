use std::num::NonZeroUsize;

use ahash::{AHashMap, AHashSet};

// An octree is a tree data structure that contains multiple "nodes" that each have 8 children
// Octrees are used for hierarchy partitionaning, terrain generation, and even collision detection
// Octrees are the 3D variant of quadtrees and quadtrees are the 2D variant of binary trees
pub struct Octree {  
    positions: Vec<vek::Vec3<i32>>,
    depths: Vec<u32>,
    max_depth: u32,
    children: Vec<Option<NonZeroUsize>>,
}

impl Octree {
    // Create an octree with a specified LOD size
    pub fn new(depth: u32) -> Self {
        Self {
            positions: Vec::new(),
            depths: Vec::new(),
            max_depth: depth,
            children: Vec::new(),
        }
    }

    // Recalculate the octree using a specific camera location
    pub fn compute(&mut self, target: vek::Vec3<f32>) -> Vec<Node> {
        // Clear vectors
        self.positions.clear();
        self.depths.clear();
        self.children.clear();

        // Keep track of the chunks we will check for
        let mut checking = vec![0usize];
        self.depths.push(0);
        self.positions.push(vek::Vec3::broadcast(-2i32.pow(self.max_depth) / 2));

        // Keep track of the generated nodes
        let mut nodes = Vec::<Node>::new();

        // Iterate over the nodes that we must evalute
        while let Some(node) = checking.pop() {
            // Get the center of the node
            let depth = self.depths[node];
            let position = self.positions[node];
            let half = vek::Vec3::broadcast(2u32.pow(self.max_depth - depth) / 2);
            let center = position + half.as_::<i32>();
            
            // Check if we should split the node into multiple
            // TODO: Find a heuristic that limits
            let split = center.as_::<f32>().distance(target) < 50.0;
        
            // Add the child nodes to check (this node became a parent node)
            if split {
                // Position offsets for children nodes
                const OFFSETS: [vek::Vec3<u32>; 8] = [
                    vek::Vec3::new(0, 0, 0),
                    vek::Vec3::new(1, 0, 0),
                    vek::Vec3::new(1, 0, 1),
                    vek::Vec3::new(0, 0, 1),
                    vek::Vec3::new(0, 1, 0),
                    vek::Vec3::new(1, 1, 0),
                    vek::Vec3::new(1, 1, 1),
                    vek::Vec3::new(0, 1, 1),
                ];

                for children in 0..8usize {
                    let pos = (OFFSETS[children] * half).as_::<i32>() + position;
                    self.depths.push(depth + 1);
                    self.children.push(None);
                    self.positions.push(pos);
                    checking.push(children + node + 1);
                }

                // We know we will *always* have 8 children, and we know they are tightly packed together
                // so instead of storing each child index we only need to store the "base" child index
                self.children[node] = Some(NonZeroUsize::new(node + 1).unwrap());
            }

            if depth != 0 {
                nodes.push(Node {
                    position,
                    depth,
                    index: node,
                    children: self.children[node],
                })
            }
        }

        nodes
    }

    // Iterate over the octree recursively using a "check" function
    // TODO: Multithread this
    pub fn recurse(&self, callback: impl Fn(Node) -> bool) {
        todo!()
    }

    // Get the size of the root node of the octree
    pub fn size(&self) -> u64 {
        2u64.pow(self.max_depth)
    } 
}

// Octree deltas contain the added / removed chunk nodes
pub struct OctreeDelta {
    pub added: Vec<Node>,
    pub removed: Vec<Node>
}

// An octree node is an object that *might* contain 8 children (it becomes a parent)
// If an octree node does not contain children, then it is considered a leaf node
pub struct Node {
    position: vek::Vec3<i32>,
    depth: u32,
    index: usize,
    children: Option<NonZeroUsize>
}

impl Node {
    // Get the world space position (bottom left near) of the node
    pub fn position(&self) -> vek::Vec3<i32> {
        self.position
    }
    
    // Get the current node's index
    pub fn index(&self) -> usize {
        self.index
    }

    // Get the depth of the node
    pub fn depth(&self) -> u32 {
        self.depth
    }
    
    // Get the node's children (if it has any)
    pub fn children(&self) -> Option<[NonZeroUsize; 8]> {
        self.children.map(|x| [
            x,
            x.saturating_add(1),
            x.saturating_add(2),
            x.saturating_add(3),

            x.saturating_add(4),
            x.saturating_add(5),
            x.saturating_add(6),
            x.saturating_add(7),
        ])
    }
}