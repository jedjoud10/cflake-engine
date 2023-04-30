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
    heuristic: OctreeHeuristic,
}

// A heuristic is what we will use to check if we should split a node or not
pub enum OctreeHeuristic {
    // Spherical heuristic with a specific radius
    Spheric(f32),

    // Point heuristic. Same as Sphere(0.0)
    Point,

    // Cube heuristic with cube full extent size
    Cubic(f32),

    // Custom heuristic
    Boxed(Box<dyn Fn(&vek::Vec3<f32>, &Node) -> bool>),
}

impl OctreeHeuristic {
    // Check if we should split a node or not
    fn check(&self, target: &vek::Vec3<f32>, node: &Node) -> bool {
        match self {
            OctreeHeuristic::Spheric(radius) => {
                crate::intersect::aabb_sphere(&node.aabb(), &crate::shapes::Sphere {
                    center: *target,
                    radius: *radius,
                })
            },
            OctreeHeuristic::Point => {
                crate::intersect::point_aabb(target, &node.aabb())
            },
            OctreeHeuristic::Cubic(extent) => {
                crate::intersect::aabb_aabb(&node.aabb(), &crate::bounds::Aabb {
                    min: target - extent / 2.0,
                    max: target + extent / 2.0,
                })
            },
            OctreeHeuristic::Boxed(func) => func(target, node),
        }
    }
}

impl Octree {
    // Create an octree with a specified LOD size and node size
    // Max depth must be greater than 2
    // Node size must be greater than 1, and it must be a power of two
    pub fn new(max_depth: u32, node_size: u32, heuristic: OctreeHeuristic) -> Self {
        assert!(max_depth > 2);
        assert!(node_size.is_power_of_two() && node_size > 1);

        Self {
            positions: Vec::new(),
            depths: Vec::new(),
            children: Vec::new(),
            max_depth,
            node_size,
            old_nodes: AHashSet::default(),
            heuristic,
        }
    }

    // Recalculate the octree using a specific camera location
    pub fn compute(&mut self, target: vek::Vec3<f32>) -> OctreeDelta {
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
            let split = self.heuristic.check(&target, &Node {
                position,
                center,
                depth,
                size: size * 2,
                leaf: true
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
                center,
                size: size * 2,
                leaf: self.children[node].is_none(),
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
                center,
                size,
                leaf: self.children[index].is_none(),
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
    size: u32,
    leaf: bool,
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

    // Get the depth of the node
    pub fn depth(&self) -> u32 {
        self.depth
    }
    
    // Check if the node is a leaf node
    pub fn leaf(&self) -> bool {
        self.leaf
    }

    // Get the AABB bounding box of this node
    pub fn aabb(&self) -> crate::Aabb<f32> {
        crate::bounds::Aabb {
            min: self.position().as_::<f32>(),
            max: self.position().as_::<f32>() + vek::Vec3::broadcast(self.size() as f32),
        }
    }
}