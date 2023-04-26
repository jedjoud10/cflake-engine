use std::num::NonZeroUsize;

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
        let mut octree = Self {
            positions: Vec::new(),
            depths: Vec::new(),
            max_depth: depth,
            children: Vec::new(),
        };

        // Add a root node to the octree
        octree.positions.push(vek::Vec3::broadcast(-2i32.pow(depth) / 2));
        octree.depths.push(0);
        octree.children.push(None);
        octree
    }

    // Recalculate the octree using a specific camera location
    // This will generate/delete nodes around the camera
    pub fn compute(&mut self, target: vek::Vec3<f32>) -> OctreeDelta {
        // Keep track of the chunks we will check for
        let mut checking = vec![0usize];

        // Iterate over the nodes that we must evalute
        while let Some(node) = checking.pop() {
            // Get the center of the node
            let depth = self.depths[node];
            let position = self.positions[node];
            let half = vek::Vec3::broadcast(2u32.pow(depth) / 2);
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
        }

        todo!()
    }

    // Iterate over the octree recursively using a "check" function
    pub fn recurse(&self, callback: impl Fn() -> bool) {
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
    parent: usize,
    index: usize,
    children: Option<[NonZeroUsize; 8]>
}

impl Node {
    // Get the world space position (bottom left near) of the node
    pub fn position(&self) -> vek::Vec3<i32> {
        self.position
    }
    
    // Get the node's parent node index
    pub fn parent(&self) -> usize {
        self.parent
    }
    
    // Get the current node's index
    pub fn index(&self) -> usize {
        self.index
    }
    
    // Get the node's children (if it has any)
    pub fn children(&self) -> Option<[NonZeroUsize; 8]> {
        self.children
    }
}