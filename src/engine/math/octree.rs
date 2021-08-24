use std::collections::HashMap;

use crate::engine::debug::DebugRendererable;

use super::shapes;
use super::Intersection;

// The octree input data
pub struct OctreeInput {
    pub camera: shapes::Sphere,
}

// The whole octree
#[derive(Default)]
pub struct Octree {
    pub nodes: HashMap<glam::IVec3, OctreeNode>,
    pub depth: u8,
}

impl Octree {
    // Generate the octree at a specific position with a specific depth
    pub fn generate_octree(&mut self, input: OctreeInput) {
        // Create the root node
        self.depth = 8;
        let root_size = 2_u16.pow(self.depth as u32) as i32;
        let root_position = glam::ivec3(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        let mut removed_nodes: Vec<OctreeNode> = Vec::new();
        let mut added_nodes: Vec<OctreeNode> = Vec::new();
        pending_nodes.push(OctreeNode { 
            position: root_position,
            extent: (root_size / 2) as u16,
            depth: 0,
            current: 0,
            parent: 0,
            children_indices: [0; 8],
            children_centers: [glam::IVec3::ZERO; 8],
            children: false,
        });
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            let extent_i32 = octree_node.extent as i32;
            // If the node contains the position, subdivide it
            let aabb = octree_node.get_aabb();            
            if Intersection::aabb_sphere(&aabb, &input.camera) && octree_node.depth < self.depth {
                // If it intersects the sphere, subdivide this octree node into multiple smaller ones
                let mut i: u16 = 0;
                for y in 0..2 {
                    for z in 0..2 {
                        for x in 0..2 {
                            // The position offset for the new octree node
                            let offset: glam::IVec3 = glam::ivec3(x *extent_i32, y * extent_i32, z * extent_i32);
                            let child = OctreeNode {
                                position: octree_node.position + offset,
                                extent: octree_node.extent / 2,
                                depth: self.depth + 1,
                                current: self.nodes.len() as u16 + i,
                                parent: octree_node.current,
                                children_indices: [0; 8],
                                children_centers: [glam::IVec3::ZERO; 8],
                                children: false,
                            };
                            let center: glam::IVec3 = child.position + glam::ivec3(child.extent as i32, child.extent as i32, child.extent as i32);
                            octree_node.children_indices[i as usize] = child.current;
                            octree_node.children_centers[i as usize] = center; 
                            pending_nodes.push(child);
                            i += 1;
                        }
                    }
                }                
                // Update the octree node
                pending_nodes[0].children = true;
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
                if !octree_node.children && last.children {
                    // Recursively get the children and put them in the removed list
                    let mut pending_children: Vec<OctreeNode> = Vec::new();
                    while pending_children.len() > 0 {
                        let current_child = pending_children[0].clone();
                        // Add the sub children if we have them
                        if current_child.children {
                            for sub_child_center in current_child.children_centers {
                                pending_children.push(self.nodes[&sub_child_center].clone());
                            }
                        }
                        // Remove the current child from the pending children and add it to the removed nodes
                        pending_children.remove(0);
                        removed_nodes.push(current_child);
                    }
                }            
            }

            // Add it to the nodes and remove it from pending nodes
            self.nodes.insert(center, octree_node);
            pending_nodes.remove(0);
        }

    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: glam::IVec3,
    pub extent: u16,
    pub depth: u8,

    // Used for the parent-children links
    pub parent: u16,
    pub current: u16,
    pub children_indices: [u16; 8],
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