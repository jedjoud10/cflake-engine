use super::HeuristicSettings;
use getset::{CopyGetters, Getters, MutGetters};
use slotmap::Key;
use std::{hash::Hash, mem::MaybeUninit, num::{NonZeroU64, NonZeroU8}};

slotmap::new_key_type! {
    pub struct NodeKey;
}

// Simple node in the octree
#[derive(Clone, Debug, Getters, CopyGetters, MutGetters)]
pub struct Node {
    // The curent position of the node. Note: Multiple nodes can have the same position, but not the same center
    #[getset(get_copy = "pub")]
    position: vek::Vec3<i64>,

    // Half extents of the node
    #[getset(get_copy = "pub")]
    half_extent: NonZeroU64,

    // Depth of the node. 0 means that is is the root node
    #[getset(get_copy = "pub")]
    depth: u8,
    
    // Parent of the node. This could be null if the node is the root node
    #[getset(get_copy = "pub")]
    parent: NodeKey,

    // The internal key index that this node uses
    #[getset(get_copy = "pub")]
    key: NodeKey,

    // Children
    #[getset(get = "pub", get_mut = "pub(super)")]
    children: Option<[NodeKey; 8]>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        // Check coordinates, then check if we have the same child count
        self.center() == other.center() && self.children.is_none() == other.children.is_none() && self.depth == other.depth
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
    pub fn root(key: NodeKey, depth: NonZeroU8, size: NonZeroU64) -> Self {
        // Get the maximum size of the root node
        let full_extent = (2_u64.pow(depth.get() as u32) * size.get()) as i64;
        let position = vek::Vec3::<i64>::new(-(full_extent / 2), -(full_extent / 2), -(full_extent / 2));

        Self {
            position,
            half_extent: NonZeroU64::new(full_extent as u64 / 2).unwrap(),
            depth: 0,
            parent: NodeKey::null(),
            key,
            children: None,
        }
    }
    // Get the AABB from this octee node
    pub fn aabb(&self) -> crate::bounds::aabb::AABB {
        crate::bounds::aabb::AABB {
            min: self.position.as_(),
            max: self.position.as_() + vek::Vec3::<f32>::broadcast(self.half_extent.get() as f32 * 2.0),
        }
    }
    // Get the center of this octree node
    pub fn center(&self) -> vek::Vec3<i64> {
        self.position + self.half_extent.get() as i64
    }
    // Check if we can subdivide this node
    pub fn can_subdivide(&self, target: &vek::Vec3<f32>, max_depth: NonZeroU8, settings: &HeuristicSettings) -> bool {
        let test = (settings.function)(self, target);
        test && self.depth < (max_depth.get() - 1)
    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide<'a>(&mut self) -> [Node; 8] {
        let half_extent = self.half_extent.get();
        const NULL: MaybeUninit<Node> = MaybeUninit::<Node>::uninit();
        let mut children = [NULL; 8];
        // Children counter
        let mut i: usize = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset = vek::Vec3::<i64>::new(x * half_extent as i64, y * half_extent as i64, z * half_extent as i64);

                    // Calculate the child's index
                    let child = Node {
                        position: self.position + offset,
                        // The children node is two times smaller in each axis
                        half_extent: NonZeroU64::new(u64::try_from(self.half_extent).unwrap() / 2).unwrap(),
                        depth: self.depth + 1,

                        // Index stuff
                        parent: self.key,
                        key: NodeKey::null(),
                        children: None,
                    };
                    unsafe {
                        children[i].as_mut_ptr().write(child);
                    }
                    i += 1;
                }
            }
        }

        unsafe { std::mem::transmute::<[MaybeUninit<Node>; 8], [Node; 8]>(children) }
    }
}
