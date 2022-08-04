use arrayvec::ArrayVec;
use slotmap::Key;
use std::{hash::Hash};

slotmap::new_key_type! {
    pub struct NodeKey;
}

// A single node within any type of octree
// A node must have a parent (except if it is the root node)
// A node *might* have 8 children
// TODO: Optimize the node's layout since it seems inefficient
#[derive(Clone, Copy, Debug)]
pub struct Node {
    // Positioning and size
    position: vek::Vec3<i64>,
    half_extent: u64,

    // Hierarchy fields
    depth: u8,
    parent: NodeKey,
    key: NodeKey,
    children: Option<[NodeKey; 8]>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.center() == other.center()
            && self.children.is_none() == other.children.is_none()
            && self.depth == other.depth
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.center().hash(state);
        self.depth.hash(state);
        self.children.is_none().hash(state);
    }
}

impl Eq for Node {}

impl Node {
    // Create a root node using the max size and depth
    pub fn root(key: NodeKey, depth: u8, size: u64) -> Self {
        // Get the maximum size of the root node
        let full_extent = (2_u64.pow(depth as u32) * size) as i64;
        let position =
            vek::Vec3::<i64>::new(-(full_extent / 2), -(full_extent / 2), -(full_extent / 2));

        Self {
            position,
            half_extent: full_extent as u64 / 2,
            depth: 0,
            parent: NodeKey::null(),
            key,
            children: None,
        }
    }

    // Get the AABB from this octee node
    pub fn aabb(&self) -> crate::AABB {
        crate::AABB {
            min: self.position.as_(),
            max: self.position.as_() + vek::Vec3::<f32>::broadcast(self.half_extent as f32 * 2.0),
        }
    }

    // Get the center of this octree node
    pub fn center(&self) -> vek::Vec3<i64> {
        self.position + self.half_extent as i64
    }

    // Get the children keys of this node
    pub fn children(&self) -> Option<&[NodeKey; 8]> {
        self.children.as_ref()
    }

    // Get the depth of this node
    pub fn depth(&self) -> u8 {
        self.depth
    }

    // Get the size of this node
    pub fn size(&self) -> u64 {
        self.half_extent * 2
    }

    // Subdivide this node into 8 smaller nodes
    pub fn subdivide<'a>(&mut self) -> [Node; 8] {
        assert!(self.children.is_none());
        let mut children = ArrayVec::<Node, 8>::new();
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset = vek::Vec3::<i64>::new(
                        x * self.half_extent as i64,
                        y * self.half_extent as i64,
                        z * self.half_extent as i64,
                    );

                    // Create the new child and add it to our vector
                    children.push(Node {
                        position: self.position + offset,
                        // The children node is two times smaller in each axis
                        half_extent: u64::try_from(self.half_extent).unwrap() / 2,
                        depth: self.depth + 1,

                        // Index stuff
                        parent: self.key,
                        key: NodeKey::null(),
                        children: None,
                    });
                }
            }
        }

        children.into_inner().unwrap()
    }
}
