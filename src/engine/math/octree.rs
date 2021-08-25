use std::collections::HashMap;
use super::shapes;
use super::Intersection;
use super::shapes::Sphere;

// The octree input data
pub struct OctreeInput {
    pub target: glam::Vec3,
}

// The whole octree
pub struct Octree {
    pub nodes: HashMap<glam::IVec3, OctreeNode>,
    pub targetted_node: Option<OctreeNode>,
    pub added_nodes: Vec<OctreeNode>,
    pub removed_nodes: Vec<OctreeNode>,
    pub threshold: f32,
    pub size: u8,
    pub depth: u8,
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            targetted_node: None,
            size: 1,
            depth: 1,
            threshold: 1.0,
        }
    }
}

impl Octree {
    // Generate an octree from a root and a target point
    pub fn generate_octree(&self, target: &glam::Vec3, root_node: OctreeNode) -> (HashMap<glam::IVec3, OctreeNode>, Option<OctreeNode>) {
        let mut nodes: HashMap<glam::IVec3, OctreeNode> = HashMap::new();
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        pending_nodes.push(root_node.clone());
        nodes.insert(root_node.get_center(), root_node);
        let mut targetted_node: Option<OctreeNode> = None; 
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            let extent_i32 = octree_node.half_extent as i32;
            // Check if this octree node is the targeted node
            if octree_node.depth == self.depth - 1 {
                targetted_node = Some(octree_node.clone());
            }
            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth) {
                // If it intersects the sphere, subdivide this octree node into multiple smaller ones
                let mut i: u16 = 0;
                for y in 0..2 {
                    for z in 0..2 {
                        for x in 0..2 {
                            // The position offset for the new octree node
                            let offset: glam::IVec3 = glam::ivec3(x * extent_i32, y * extent_i32, z * extent_i32);
                            let child = OctreeNode {
                                position: octree_node.position + offset,
                                half_extent: octree_node.half_extent / 2,
                                depth: octree_node.depth + 1,
                                parent_center: octree_node.get_center(),
                                children_centers: [glam::IVec3::ZERO; 8],
                                children: false,
                            };                            
                            let center = child.get_center();
                            octree_node.children_centers[i as usize] = center; 
                            pending_nodes.push(child);                            
                            i += 1;
                        }
                    }
                }                
                // Update the octree node
                octree_node.children = true;
            }
            // Bruh
            pending_nodes.remove(0);
            nodes.insert(octree_node.get_center(), octree_node);
        }
        return (nodes, targetted_node);
    }
    // Generate the base octree with a target point at 0, 0, 0
    pub fn generate_base_octree(&mut self) {
        let root_size = (2_u32.pow(self.depth as u32) * self.size as u32) as i32;
        let root_position = glam::ivec3(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        // Create the root node
        let root_node = OctreeNode { 
            position: root_position,
            half_extent: (root_size / 2) as u16,
            depth: 0,
            parent_center: glam::IVec3::ZERO,
            children_centers: [glam::IVec3::ZERO; 8],
            children: false,
        };
        let octree_data = self.generate_octree(&glam::Vec3::ONE, root_node);
        self.nodes = octree_data.0;
        self.targetted_node = octree_data.1;
        println!("{:?}", self.targetted_node);
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(&mut self, input: OctreeInput) {
        // If we don't have a targetted node, exit early
        if self.targetted_node.is_none() { return; }
        let marked_node: Option<OctreeNode>;
        // Go up the tree, marking the nodes that have been removed along the way
        {
            let mut current_node: OctreeNode = self.targetted_node.clone().unwrap();
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            let targetted_node = self.targetted_node.clone().unwrap();
            pending_nodes.push(targetted_node);
            // Loop until you can subdivide
            while !current_node.can_subdivide(&input.target, self.depth) {
                // Set the current node as the current's node parent
                current_node = self.nodes.get(&current_node.parent_center).unwrap().clone();

                // If we are the root node, exit since we are sure that there must be an intersection (If the target is inside the octree that is)
                if current_node.depth == 0 {
                    break;
                }
            }
            println!("Max depth: {}", current_node.depth);
            // We did find an intersection
            marked_node = Some(current_node);            
        }
        // Not good
        if marked_node.is_none() { return; }
        // Then we generate a local octree, using that marked node as the root
        let local_octree_data = self.generate_octree(&input.target, marked_node.unwrap());
        //self.targetted_node = local_octree_data.1;
        let added_nodes = local_octree_data.0;
        self.added_nodes = added_nodes.values().map(|x| x.clone()).collect();
    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: glam::IVec3,
    pub half_extent: u16,
    pub depth: u8,


    // Used for the parent-children links
    // TODO: Change this to it uses IDs instead of coordinates
    pub parent_center: glam::IVec3,
    pub children_centers: [glam::IVec3; 8],
    // Check if we had children
    pub children: bool,
}

impl OctreeNode {
    // Get the AABB from this octee node
    pub fn get_aabb(&self) -> super::bounds::AABB {
        super::bounds::AABB {
            min: self.position.as_f32(),
            max: self.position.as_f32() + glam::vec3(self.half_extent as f32, self.half_extent as f32, self.half_extent as f32) * 2.0,
        }
    }
    // Get the center of this octree node
    pub fn get_center(&self) -> glam::IVec3 {
        return self.position + self.half_extent as i32;
    }
    // Check if we can subdivide this node
    pub fn can_subdivide(&self, target: &glam::Vec3, max_depth: u8) -> bool {
        return Intersection::point_aabb(target, &self.get_aabb()) && self.depth < (max_depth - 1);
    }
}