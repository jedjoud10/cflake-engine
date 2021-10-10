// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,
    // Check if we had children
    pub children: bool,
    // Our children's ids
    pub children_centers: [u32; 8],
}

impl OctreeNode {
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> super::bounds::AABB {
        super::bounds::AABB {
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
        let aabb = self.get_aabb().min.elem_lte(target) & self.get_aabb().max.elem_gt(target);
        let aabb = (aabb | veclib::Vector3::<bool>::new(false, false, false)).all();
        return aabb && self.depth < (max_depth - 1);
    }
    // Check if we can subdivide this node during the postprocessing loop
    pub fn can_subdivide_postprocess(&self, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let mut aabb = self.get_aabb();
        aabb.expand(lod_factor * self.half_extent as f32);
        let aabb = aabb.min.elem_lte(target) & aabb.max.elem_gt(target);
        let aabb = (aabb).all();
        return aabb && self.depth < (max_depth - 1);
    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide(&mut self) -> Vec<OctreeNode> {
        let extent_i64 = self.half_extent as i64;
        let mut output: Vec<OctreeNode> = Vec::new();
        let mut i: u8 = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: veclib::Vector3<i64> = veclib::Vector3::<i64>::new(x * extent_i64, y * extent_i64, z * extent_i64);
                    let mut new_path = self.path.clone();
                    new_path.push(i);
                    let child = OctreeNode {
                        position: self.position + offset,
                        half_extent: self.half_extent / 2,
                        depth: self.depth + 1,
                        parent_center: self.get_center(),
                        postprocess: false,
                        children_centers: [veclib::Vector3::<i64>::ZERO; 8],
                        child_leaf_count: 0,
                        children: false,
                        path: new_path,
                    };
                    let center = child.get_center();
                    self.children_centers[i as usize] = center;
                    output.push(child);
                    i += 1;
                }
            }
        }
        // Update the octree node
        self.children = true;
        return output;
    }
}
