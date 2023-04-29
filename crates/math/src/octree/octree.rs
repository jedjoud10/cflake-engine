use std::num::NonZeroUsize;
use ahash::{AHashSet};

// An octree is a tree data structure that contains multiple "nodes" that each have 8 children
// Octrees are used for hierarchy partitionaning, terrain generation, and even collision detection
// Octrees are the 3D variant of quadtrees and quadtrees are the 2D variant of binary trees
pub struct Octree {  
    positions: Vec<vek::Vec3<i32>>,
    depths: Vec<u32>,
    max_depth: u32,
    node_size: u32,
    children: Vec<Option<NonZeroUsize>>,
    old_nodes: AHashSet<Node>,
}

impl Octree {
    // Create an octree with a specified LOD size and node size
    // Max depth must be greater than 2
    // Node size must be greater than 1, and it must be a power of two
    pub fn new(max_depth: u32, node_size: u32) -> Self {
        assert!(max_depth > 2);
        assert!(node_size.is_power_of_two() && node_size > 1);

        Self {
            positions: Vec::new(),
            depths: Vec::new(),
            children: Vec::new(),
            max_depth,
            node_size,
            old_nodes: AHashSet::default(),
        }
    }

    // Recalculate the octree using a specific camera location
    pub fn compute(&mut self, target: vek::Vec3<f32>, radius: f32) -> OctreeDelta {
        // Clear vectors
        self.positions.clear();
        self.depths.clear();
        self.children.clear();

        // Needed for diff
        let mut new = AHashSet::<Node>::new();

        // Keep track of the chunks we will check for
        let mut checking = vec![0usize];
        self.depths.push(0);
        self.children.push(None);
        self.positions.push(vek::Vec3::broadcast((-2i32.pow(self.max_depth) * self.node_size as i32) / 2));

        // Iterate over the nodes that we must evalute
        while let Some(node) = checking.pop() {
            // Get the center of the node
            let depth = self.depths[node];
            let position = self.positions[node];
            let size = (2u32.pow(self.max_depth - depth) * self.node_size) / 2;
            let half = vek::Vec3::broadcast(size);
            let center = position + half.as_::<i32>();

            // Check if we should split the node into multiple
            // TODO: Find a heuristic that limits
            let split = crate::intersect::aabb_sphere(&crate::bounds::Aabb {
                min: position.as_::<f32>(),
                max: position.as_::<f32>() + half.as_::<f32>() * 2.0,
            }, &crate::shapes::Sphere {
                center: target,
                radius,
            });

            // Add the child nodes to check (this node became a parent node)
            if split && depth < self.max_depth {
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

                // Add the children to the nodes that we must process
                let base = self.positions.len();
                checking.extend(base..(base + 8));

                // Create the children nodes and add them to the octree                 
                for children in 0..8usize {
                    let pos = (OFFSETS[children] * half).as_::<i32>() + position;
                    self.depths.push(depth + 1);
                    self.children.push(None);
                    self.positions.push(pos);
                }

                // We know we will *always* have 8 children, and we know they are tightly packed together
                // so instead of storing each child index we only need to store the "base" child index
                self.children[node] = Some(NonZeroUsize::new(base).unwrap());
            }

            // Add a node to the new octree nodes
            new.insert(Node {
                position,
                depth,
                index: node,
                children: self.children[node],
                center,
                size: (2u32.pow(self.max_depth - depth) * self.node_size),
            });
        }

        let previous = std::mem::take(&mut self.old_nodes);
        let current = new;

        // And check for differences
        let removed = previous
            .difference(&current)
            .cloned()
            .collect::<Vec<_>>();
        let added = current
            .difference(&previous)
            .cloned()
            .collect::<Vec<_>>();
        self.old_nodes = current;

        OctreeDelta {
            added,
            removed
        }
    }

    // Iterate over the octree recursively using a "check" function
    pub fn recurse(&self, callback: impl Fn(Node) -> bool) {
        let mut checking = vec![0usize];
        while let Some(index) = checking.pop() {
            let depth = self.depths[index];
            let position = self.positions[index];
            let size = (2u32.pow(self.max_depth - depth) * self.node_size) / 2;
            let half = vek::Vec3::broadcast(size);
            let center = position + half.as_::<i32>();
            let children = self.children[index].is_some();

            let node = Node {
                position,
                depth,
                index,
                children: self.children[index],
                center,
                size,
            };

            if children && callback(node) {
                let base = index + 1;
                checking.extend(base..(base + 8));
            }
        }
    }

    // Get the size of the root node of the octree
    pub fn size(&self) -> u64 {
        2u64.pow(self.max_depth) * self.node_size as u64
    } 
}

// Octree deltas contain the added / removed chunk nodes
pub struct OctreeDelta {
    pub added: Vec<Node>,
    pub removed: Vec<Node>
}

// An octree node is an object that *might* contain 8 children (it becomes a parent)
// If an octree node does not contain children, then it is considered a leaf node
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Node {
    position: vek::Vec3<i32>,
    center: vek::Vec3<i32>,
    depth: u32,
    index: usize,
    size: u32,
    children: Option<NonZeroUsize>
}

impl Node {
    // Get the world space position (bottom left near) of the node
    pub fn position(&self) -> vek::Vec3<i32> {
        self.position
    }

    // Get the center of the Node
    pub fn center(&self) -> vek::Vec3<i32> {
        self.center
    }

    // Get the full extent size of the node
    pub fn size(&self) -> u32 {
        self.size
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