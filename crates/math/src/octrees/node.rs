use std::{collections::HashSet, hash::Hash};
use others::SmartList;

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
        return self.get_center() == other.get_center() && self.children_indices.is_none() == other.children_indices.is_none() && self.depth == other.depth; 
    }
}

impl Hash for OctreeNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_center().hash(state);
        self.children_indices.is_none().hash(state);
    }
}

impl Eq for OctreeNode {
}

impl OctreeNode {
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> crate::bounds::AABB {
        crate::bounds::AABB {
            min: veclib::Vector3::<f32>::from(self.position),
            max: veclib::Vector3::<f32>::from(self.position) + veclib::Vector3::<f32>::new(self.half_extent as f32, self.half_extent as f32, self.half_extent as f32) * 2.0,
        }
    }
    // Get the center of this octree node
    pub fn get_center(&self) -> veclib::Vector3<i64> {
        return self.position + self.half_extent as i64;
    }
    // Check if we can subdivide this node
    pub fn can_subdivide(&self, target: &veclib::Vector3<f32>, max_depth: u8) -> bool {
        // AABB intersection, return true if point in on the min edge though
        let aabb = (self.get_aabb().min.elem_lte(target) & self.get_aabb().max.elem_gt(target)).all();
        return aabb && self.depth < (max_depth - 1);
    }
    // Recursively find the children for this node
    pub fn find_children_recursive(&self, nodes: &SmartList<OctreeNode>) -> Vec<OctreeNode> {
        let mut list: Vec<OctreeNode> = Vec::new();
        let mut pending: Vec<OctreeNode> = vec![self.clone()];

        while pending.len() > 0 {
            // Get the current node to evaluate
            let current = pending.get(0).unwrap().clone();
            // Add children
            match current.children_indices {
                Some(x) => {
                    // Add them
                    pending.extend(x.iter().map(|index| nodes.get_element(*index).unwrap().unwrap().clone()));
                },
                None => {},
            }

            // A
            pending.remove(0);
            if current.index != self.index { list.push(current.clone()); }
        }
        return list;

    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide(&mut self, nodes: &mut SmartList<OctreeNode>) -> Vec<OctreeNode> {
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
                    let child_index = nodes.get_next_valid_id();                    

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
                    nodes.add_element(child);
                    i += 1;
                }
            }
        }

        // Update the children indices
        self.children_indices = Some(children_indices);

        // Update the parent node
        let elm = nodes.get_element_mut(self.index).unwrap().unwrap();
        elm.children_indices = Some(children_indices);

        return output;
    }
}
