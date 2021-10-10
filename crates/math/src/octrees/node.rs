// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,
    // Indexing stuff
    pub parent_index: u32,
    pub index: u32,
    pub children_indices: Option<[u32; 8]>,
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
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide(&mut self) -> Vec<OctreeNode> {
        let half_extent = self.half_extent as i64;
        // The outputted nodes
        let mut output: Vec<OctreeNode> = Vec::new();

        // Temporary array that we fill with out children's indices
        let mut children_indices: [u32; 8];

        // Children counter
        let mut i: usize = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: veclib::Vector3<i64> = veclib::Vector3::<i64>::new(x * half_extent, y * half_extent, z * half_extent);

                    // Calculate the child's index
                    let child_index = self.index * 8 + (i as u32);

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
                    let center = child.get_center();
                    // Update the indices
                    children_indices[i] = child_index;
                    output.push(child);
                    i += 1;
                }
            }
        }

        // Turn our children indices to an empty 8 element array
        self.children_indices = Some(children_indices);
        return output;
    }
}
