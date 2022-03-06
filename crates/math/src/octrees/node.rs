use super::HeuristicSettings;
use slotmap::{Key, SlotMap};
use std::{hash::Hash, mem::MaybeUninit};

slotmap::new_key_type! {
    pub struct NodeKey;
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct Node {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,
    // Indexing stuff
    pub parent_key: NodeKey,
    pub key: NodeKey,
    pub children_keys: Option<[NodeKey; 8]>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        // Check coordinates, then check if we have the same child count
        self.center() == other.center() && self.children_keys.is_none() == other.children_keys.is_none() && self.depth == other.depth
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.center().hash(state);
        self.depth.hash(state);
        self.children_keys.is_none().hash(state);
    }
}

impl Eq for Node {}

impl Node {
    // Get the AABB from this octee node
    pub fn aabb(&self) -> crate::bounds::aabb::AABB {
        crate::bounds::aabb::AABB {
            min: veclib::Vector3::<f32>::from(self.position),
            max: veclib::Vector3::<f32>::from(self.position) + veclib::Vector3::<f32>::new(self.half_extent as f32, self.half_extent as f32, self.half_extent as f32) * 2.0,
        }
    }
    // Get the center of this octree node
    pub fn center(&self) -> veclib::Vector3<i64> {
        self.position + self.half_extent as i64
    }
    // Check if we can subdivide this node
    pub fn can_subdivide(&self, target: &veclib::Vector3<f32>, max_depth: u8, settings: &HeuristicSettings) -> bool {
        let test = (settings.function)(self, target);
        test && self.depth < (max_depth - 1)
    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide<'a>(&mut self) -> [Node; 8] {
        let half_extent = self.half_extent as i64;
        const NULL: MaybeUninit<Node> = MaybeUninit::<Node>::uninit();
        let mut children = [NULL; 8];
        // Children counter
        let mut i: usize = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: veclib::Vector3<i64> = veclib::Vector3::<i64>::new(x * half_extent, y * half_extent, z * half_extent);

                    // Calculate the child's index
                    let child = Node {
                        position: self.position + offset,
                        // The children node is two times smaller in each axis
                        half_extent: self.half_extent / 2,
                        depth: self.depth + 1,

                        // Index stuff
                        parent_key: self.key,
                        key: NodeKey::null(),
                        children_keys: None,
                    };
                    unsafe {
                        children[i].as_mut_ptr().write(child);
                    }
                    i += 1;
                }
            }
        }
        let result = unsafe { std::mem::transmute::<[MaybeUninit<Node>; 8], [Node; 8]>(children) };
        result
    }
}
