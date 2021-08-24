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
    pub nodes: Vec<OctreeNode>,
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
        pending_nodes.push(OctreeNode { position: root_position, extent: (root_size / 2) as u16, depth: 0 });
        
        // Add all the other octree nodes
        while pending_nodes.len() > 0 {
            let octree_node = &pending_nodes[0];
            // If the node contains the position, subdivide it
            let aabb = octree_node.get_aabb();
            if Intersection::aabb_sphere(&aabb, &input.camera) && octree_node.depth < self.depth {
                // If it intersects the sphere, subdivide this octree node into multiple smaller ones
                let mut children_nodes = octree_node.subdivide();
                pending_nodes.append(&mut children_nodes);
            } else if octree_node.depth < self.depth {
                // Add the node if it has no children
                self.nodes.push(octree_node.clone());
            }
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
}

impl OctreeNode {
    // Subdivide the current node into 8 new nodes
    pub fn subdivide(&self) -> Vec<OctreeNode> {
        let mut children: Vec<OctreeNode> = Vec::new();        
        // Subdivide
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: glam::IVec3 = glam::ivec3(x * self.extent as i32, y * self.extent as i32, z * self.extent as i32);
                    let temp_octree_node = OctreeNode {
                        position: self.position + offset,
                        extent: self.extent / 2,
                        depth: self.depth + 1,
                    };
                    children.push(temp_octree_node);
                }
            }
        }
        children
    } 
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> super::bounds::AABB {
        super::bounds::AABB {
            min: self.position.as_f32(),
            max: self.position.as_f32() + glam::vec3(self.extent as f32, self.extent as f32, self.extent as f32) * 2.0,
        }
    }
}