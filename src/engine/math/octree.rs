use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
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
    pub size: u32,
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
        //let target = ((target_n.as_i32() - glam::ivec3(self.size as i32, self.size as i32, self.size as i32)).as_f32() / self.size as f32).round() * self.size as f32 + (self.size as f32 / 2.0);
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
            half_extent: (root_size / 2) as u32,
            depth: 0,
            parent_center: glam::IVec3::ZERO,
            children_centers: [glam::IVec3::ZERO; 8],
            children: false,
        };
        let octree_data = self.generate_octree(&glam::Vec3::ONE, root_node);
        self.nodes = octree_data.0;
        self.targetted_node = octree_data.1;
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(&mut self, input: OctreeInput) {
        let instant = Instant::now();
        self.added_nodes.clear();
        self.removed_nodes.clear();        
        // If we don't have a targetted node, exit early
        if self.targetted_node.is_none() { return; }
        let marked_node: Option<OctreeNode>;
        
        // We'll have only one main octree node that we will remove, and we will recursively remove it's children as well
        let mut node_to_remove: Option<OctreeNode> = None;
        // Go up the tree, marking the nodes that have been removed along the way
        {
            let mut current_node: OctreeNode = self.targetted_node.clone().unwrap();
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            let targetted_node = self.targetted_node.clone().unwrap();
            let mut intersection: bool = false;
            pending_nodes.push(targetted_node);
            // Loop until you can subdivide
            while !intersection {  
                // Set the current node as the current's node parent
                current_node = self.nodes.get(&current_node.parent_center).unwrap().clone();  
                // Test for intersection 
                intersection = current_node.can_subdivide(&input.target, self.depth);
                // If it doesn't hit, then that node must be removed
                if !intersection {
                    // Since we are moving up the tree, we will get rid of this node and all of it's children
                    if current_node.children {                        
                        node_to_remove = Some(current_node.clone());
                    }
                } else {
                    // It hit
                    break;
                }
                // If we are the root node, exit since we are sure that there must be an intersection (If the target is inside the octree that is)
                if current_node.depth == 0 {
                    break;
                }
            }
            // We did find an intersection
            marked_node = Some(current_node);            
        }        
        // Check if we even changed parents
        if marked_node.is_none() || node_to_remove.is_none() { return; }
        // Then we generate a local octree, using that marked node as the root
        let local_octree_data = self.generate_octree(&input.target, marked_node.clone().unwrap());
        self.targetted_node = local_octree_data.1;
        // Get the nodes that we've added
        let added_nodes = local_octree_data.0;            

        // Set the added nodes
        self.added_nodes = added_nodes.values().map(|x| x.clone()).filter(|x| {    
            let valid_in_nodes = self.nodes.get(&x.get_center());
            match valid_in_nodes {
                Some(_) => { false }
                None => { true }
            }
        }).collect();

        // Add the delta to the nodes
        self.nodes.extend(added_nodes.clone());

        // Get the nodes that we've deleted
        let mut deleted_centers: HashSet<glam::IVec3> = HashSet::new();
        {    
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            pending_nodes.push(node_to_remove.clone().unwrap());
            // Recursively delete the nodes
            let mut i = 0;
            while pending_nodes.len() > 0 {
                let current_node = pending_nodes[0].clone();
                // Just in case
                if current_node.children {
                    // Get the children
                    for child_center in current_node.children_centers {
                        if child_center != glam::IVec3::ZERO {
                            let child_node = self.nodes.get(&child_center).unwrap().clone();
                            pending_nodes.push(child_node);
                        }
                    }
                }
                deleted_centers.insert(current_node.get_center());
                pending_nodes.remove(0);
                i += 1;
            }
        }
        
        // Update the removed node
        let mut node_to_remove = node_to_remove.unwrap();
        node_to_remove.children = false;
        node_to_remove.children_centers = [glam::IVec3::ZERO; 8];        
        self.added_nodes.push(node_to_remove.clone());
        self.nodes.insert(node_to_remove.get_center(), node_to_remove.clone());    
        
        let center: glam::IVec3 = marked_node.as_ref().unwrap().get_center();
        let depth: u8 = marked_node.as_ref().unwrap().depth;
        
        println!("Time in micros: {}", instant.elapsed().as_micros());
        // Remove the nodes
        self.removed_nodes = self.nodes.iter().filter_map(|(&coord, node)| {
            // Check if this node should be removed
            let mut valid: bool = node.children || center == coord;
            if depth != 0 {
                valid |= node.parent_center == center && node.can_subdivide(&input.target, self.depth);
            }
            if deleted_centers.contains(&coord) && coord != node_to_remove.get_center() || valid  {
                Some(node.clone())
            } else { None }
        }).collect();        
        self.nodes.retain(|k, _| !deleted_centers.contains(k) || *k == node_to_remove.get_center());
    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: glam::IVec3,
    pub half_extent: u32,
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
        // AABB intersection, return true if point in on the min edge though
        let aabb = self.get_aabb().min.cmple(*target).all() && self.get_aabb().max.cmpgt(*target).all();
        return aabb && self.depth < (max_depth - 1);
    }
}