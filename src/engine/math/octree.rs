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
        self.nodes.push(OctreeNode { position: root_position, extent: (root_size / 2) as u16, depth: 0 });
        
        // Add all the other octree nodes
        for octree_node_index in 0..self.nodes.len() {
            let octree_node = self.nodes.get(octree_node_index).unwrap();
            // If the node contains the position, subdivide it
            let aabb = octree_node.get_aabb();
            if Intersection::aabb_sphere(&aabb, &input.camera) {
                // If it intersects the sphere, subdivide this octree node into multiple smaller ones
                let mut children_nodes = octree_node.subdivide();
                self.nodes.append(&mut children_nodes);
            }
        }
    }
}

// Simple node in the octree
#[derive(Debug)]
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
        for y in 0..1 {
            for z in 0..1 {
                for x in 0..1 {
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
            max: self.position.as_f32() + glam::vec3(self.extent as f32, self.extent as f32, self.extent as f32),
        }
    }
}