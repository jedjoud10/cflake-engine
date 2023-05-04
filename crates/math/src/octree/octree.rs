use std::{num::NonZeroUsize, hash::Hash, mem::MaybeUninit};
use ahash::{AHashSet};

// An octree is a tree data structure that contains multiple "nodes" that each have 8 children
// Octrees are used for hierarchy partitionaning, terrain generation, and even collision detection
// Octrees are the 3D variant of quadtrees and quadtrees are the 2D variant of binary trees
pub struct Octree {  
    max_depth: u32,
    node_size: u32,
    nodes: Vec<Node>,
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
            max_depth,
            node_size,
            nodes: Vec::default(),
            old_nodes: AHashSet::default(),
            heuristic,
        }
    }

    // Recalculate the octree using a specific camera location
    pub fn compute(&mut self, target: vek::Vec3<f32>) -> OctreeDelta {
        self.nodes.clear();
        
        // Keep track of the chunks we will check for
        let mut checking = vec![0usize];
        self.nodes.push(Node {
            index: 0,
            position: vek::Vec3::broadcast((-2i32.pow(self.max_depth) * self.node_size as i32) / 2),
            center: vek::Vec3::zero(),
            depth: 0,
            size: (2u32.pow(self.max_depth) * self.node_size) / 2,
            children: None,
        });

        // Iterate over the nodes that we must evalute
        while let Some(node) = checking.pop() {
            // Get the center of the node
            let base = self.nodes.len();
            let node = &mut self.nodes[node];

            // Check if we should split the node into multiple
            let split = self.heuristic.check(&target, &*node);

            // Add the child nodes to check (this node became a parent node)
            let children = if split && node.depth < self.max_depth {
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
                checking.extend(base..(base + 8));

                // Create the children nodes and add them to the octree       
                let size = (2u32.pow(self.max_depth - node.depth) * self.node_size) / 2;
                let half = vek::Vec3::broadcast(size);
                let position = node.position;
                let depth = node.depth;

                let children = (0..8usize).into_iter().map(move |children| {
                    let position = (OFFSETS[children] * half).as_::<i32>() + position;
                
                    Node {
                        index: children + base,
                        position,
                        center: position + (half.as_::<i32>() / 2),
                        depth: depth + 1,
                        size,
                        children: None,
                    }
                });

                // We know we will *always* have 8 children, and we know they are tightly packed together
                // so instead of storing each child index we only need to store the "base" child index
                node.children = Some(NonZeroUsize::new(base).unwrap());
                Some(children)
            } else {
                None
            };

            drop(node);
            
            if let Some(children) = children {
                self.nodes.extend(children);
            }
        }

        // Convert vector into hashset
        let new = self.nodes.iter().cloned().collect::<AHashSet<_>>();

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
        todo!()
    }

    // Get the size of the root node of the octree
    pub fn size(&self) -> u64 {
        2u64.pow(self.max_depth) * self.node_size as u64
    }

    // Get the internally stored nodes
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    } 
}

// Octree deltas contain the added / removed chunk nodes
pub struct OctreeDelta {
    pub added: Vec<Node>,
    pub removed: Vec<Node>
}

// An octree node is an object that *might* contain 8 children (it becomes a parent)
// If an octree node does not contain children, then it is considered a leaf node
#[derive(Clone, Copy, Eq, Debug)]
pub struct Node {
    index: usize,
    position: vek::Vec3<i32>,
    center: vek::Vec3<i32>,
    depth: u32,
    size: u32,
    children: Option<NonZeroUsize>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.children.is_some() == other.children.is_some()
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.center.hash(state);
        self.children.is_some().hash(state);
    }
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
        self.children.is_none()
    }

    // Get the child base index
    pub fn children(&self) -> Option<NonZeroUsize> {
        self.children
    }

    // Get the AABB bounding box of this node
    pub fn aabb(&self) -> crate::Aabb<f32> {
        crate::bounds::Aabb {
            min: self.position().as_::<f32>(),
            max: self.position().as_::<f32>() + vek::Vec3::broadcast(self.size() as f32),
        }
    }
}