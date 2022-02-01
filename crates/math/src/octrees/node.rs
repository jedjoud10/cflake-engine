use ordered_vec::simple::UnversionnedOrderedVec;
use std::hash::Hash;
use super::HeuristicSettings;

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,
    // Indexing stuff
    pub parent_index: usize,
    pub index: usize,
    pub children_indices: Option<[usize; 8]>,
}

impl PartialEq for OctreeNode {
    fn eq(&self, other: &Self) -> bool {
        // Check coordinates, then check if we have the same child count
        self.get_center() == other.get_center() && self.children_indices.is_none() == other.children_indices.is_none() && self.depth == other.depth
    }
}

impl Hash for OctreeNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_center().hash(state);
        self.depth.hash(state);
        self.children_indices.is_none().hash(state);
    }
}

impl Eq for OctreeNode {}

impl OctreeNode {
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> crate::bounds::AABB {
        crate::bounds::AABB {
            min: veclib::Vector3::<f32>::from(self.position),
            max: veclib::Vector3::<f32>::from(self.position) + veclib::Vector3::<f32>::new(self.half_extent as f32, self.half_extent as f32, self.half_extent as f32) * 2.0,
            center: self.get_center().into(),
        }
    }
    // Get the center of this octree node
    pub fn get_center(&self) -> veclib::Vector3<i64> {
        self.position + self.half_extent as i64
    }
    // Check if we can subdivide this node
    pub fn can_subdivide(&self, target: &veclib::Vector3<f32>, max_depth: u8, settings: &HeuristicSettings) -> bool {
        let test = (settings.function)(self, target);
        test && self.depth < (max_depth - 1)
    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide(&mut self, nodes: &mut UnversionnedOrderedVec<OctreeNode>) -> Vec<OctreeNode> {
        let half_extent = self.half_extent as i64;
        // The outputted nodes
        let mut output: Vec<OctreeNode> = Vec::new();

        // Temporary array that we fill with out children's indices
        let mut children_indices: [usize; 8] = [0; 8];

        // Children counter
        let mut i: usize = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: veclib::Vector3<i64> = veclib::Vector3::<i64>::new(x * half_extent, y * half_extent, z * half_extent);

                    // Calculate the child's index
                    let child_index = nodes.get_next_idx();

                    let child = OctreeNode {
                        position: self.position + offset,
                        // The children node is two times smaller in each axis
                        half_extent: self.half_extent / 2,
                        depth: self.depth + 1,

                        // Index stuff
                        parent_index: self.index,
                        index: child_index,
                        children_indices: None,
                    };
                    // Update the indices
                    children_indices[i] = child_index;
                    output.push(child.clone());
                    nodes.push_shove(child);
                    i += 1;
                }
            }
        }

        // Update the children indices
        self.children_indices = Some(children_indices);

        // Update the parent node
        let elm = nodes.get_mut(self.index).unwrap();
        elm.children_indices = Some(children_indices);

        output
    }
}
