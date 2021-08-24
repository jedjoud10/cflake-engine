use std::collections::HashMap;
use super::shapes;
use super::Intersection;

// The octree input data
pub struct OctreeInput {
    pub camera: shapes::Sphere,
}

// The whole octree
pub struct Octree {
    pub nodes: HashMap<glam::IVec3, OctreeNode>,
    pub added_nodes: Vec<OctreeNode>,
    pub removed_nodes: Vec<OctreeNode>,
    pub size: u8,
    pub depth: u8,
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            size: 1,
            depth: 1,
        }
    }
}

impl Octree {
    // Generate the octree at a specific position with a specific depth
    pub fn generate_octree(&mut self, input: OctreeInput) {
        // Create the root node
        let root_size = (2_u32.pow(self.depth as u32) * self.size as u32) as i32;
        let root_position = glam::ivec3(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        let mut removed_nodes: Vec<OctreeNode> = Vec::new();
        let mut added_nodes: Vec<OctreeNode> = Vec::new();
        let mut local_nodes: HashMap<glam::IVec3, OctreeNode> = HashMap::new();
        pending_nodes.push(OctreeNode { 
            position: root_position,
            extent: (root_size / 2) as u16,
            depth: 0,
            children_centers: [glam::IVec3::ZERO; 8],
            children: false,
        });
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            let extent_i32 = octree_node.extent as i32;
            // If the node contains the position, subdivide it
            let aabb = octree_node.get_aabb();            
            if Intersection::point_aabb(&input.camera.center, &aabb) && octree_node.depth < (self.depth - 1) {
                // If it intersects the sphere, subdivide this octree node into multiple smaller ones
                let mut i: u16 = 0;
                for y in 0..2 {
                    for z in 0..2 {
                        for x in 0..2 {
                            // The position offset for the new octree node
                            let offset: glam::IVec3 = glam::ivec3(x * extent_i32, y * extent_i32, z * extent_i32);
                            let child = OctreeNode {
                                position: octree_node.position + offset,
                                extent: octree_node.extent / 2,
                                depth: octree_node.depth + 1,
                                children_centers: [glam::IVec3::ZERO; 8],
                                children: false,
                            };
                            let center: glam::IVec3 = child.position + glam::ivec3(child.extent as i32, child.extent as i32, child.extent as i32);
                            octree_node.children_centers[i as usize] = center; 
                            pending_nodes.push(child);
                            i += 1;
                        }
                    }
                }                
                // Update the octree node
                octree_node.children = true;
            }
            // If we don't have the current node in the last run nodes, that means that we've added it
            let center: glam::IVec3 = octree_node.position + glam::ivec3(octree_node.extent as i32, octree_node.extent as i32, octree_node.extent as i32);
            if !self.nodes.contains_key(&center) {
                // This means that this is a new node
                added_nodes.push(octree_node.clone());
            } else {
                // This node did not change / Was removed
                // If we currently don't have children and we had them in the last run, that means that we've removed them
                let last = self.nodes.get(&center).unwrap();
                // Since the current node isn't a parent any more, add it as a genuine node
                added_nodes.push(octree_node.clone());
                if !octree_node.children && last.children {
                    // Recursively get the children and put them in the removed list
                    let mut pending_children: Vec<OctreeNode> = Vec::new();
                    for default_sub_child_center in last.children_centers {
                        // Make sure it's valid (Well we technically do use the 0, 0, 0 center for the root node but we're never gonna have the root as a child so )
                        if default_sub_child_center != glam::IVec3::ZERO {
                            pending_children.push(self.nodes[&default_sub_child_center].clone());
                        }
                    }
                    while pending_children.len() > 0 {
                        let current_child = pending_children[0].clone();
                        // Add the sub children if we have them
                        if current_child.children {
                            for sub_child_center in current_child.children_centers {
                                // Same reason as the other
                                if sub_child_center != glam::IVec3::ZERO {
                                    pending_children.push(self.nodes[&sub_child_center].clone());
                                }
                            }
                        }
                        // Remove the current child from the pending children and add it to the removed nodes
                        pending_children.remove(0);
                        removed_nodes.push(current_child);
                    }
                }       
            }
            // Add it to the nodes and remove it from pending nodes
            local_nodes.insert(center, octree_node);
            pending_nodes.remove(0);
        }
        // Update self
        self.nodes.clear();
        self.nodes.extend(local_nodes);
        self.added_nodes = added_nodes;
        self.removed_nodes = removed_nodes;        
    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: glam::IVec3,
    pub extent: u16,
    pub depth: u8,

    // Used for the parent-children links
    pub children_centers: [glam::IVec3; 8],
    // Check if we had children
    pub children: bool,
}

impl OctreeNode {
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> super::bounds::AABB {
        super::bounds::AABB {
            min: self.position.as_f32(),
            max: self.position.as_f32() + glam::vec3(self.extent as f32, self.extent as f32, self.extent as f32) * 2.0,
        }
    }
}