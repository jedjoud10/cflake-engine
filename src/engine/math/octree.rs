use super::shapes;
use super::shapes::Sphere;
use super::Intersection;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

// The octree input data
pub struct OctreeInput {
    pub target: veclib::Vector3<f32>,
}

// The whole octree
pub struct Octree {
    pub nodes: HashMap<veclib::Vector3<i64>, OctreeNode>,
    pub targetted_node: Option<OctreeNode>,
    pub added_nodes: Vec<OctreeNode>,
    pub removed_nodes: Vec<OctreeNode>,
    pub final_added_nodes: Vec<OctreeNode>,
    pub final_removed_nodes: Vec<OctreeNode>,
    pub final_nodes: HashMap<veclib::Vector3<i64>, OctreeNode>,
    pub lod_factor: f32,
    pub size: u64,
    pub depth: u8,
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            final_nodes: HashMap::new(),
            final_added_nodes: Vec::new(),
            final_removed_nodes: Vec::new(),
            targetted_node: None,
            size: 1,
            depth: 1,
            lod_factor: 1.0,
        }
    }
}

impl Octree {
    // Generate an octree from a root and a target point
    pub fn generate_octree(&self, target: &veclib::Vector3<f32>, root_node: OctreeNode) -> (HashMap<veclib::Vector3<i64>, OctreeNode>, Option<OctreeNode>) {
        let mut nodes: HashMap<veclib::Vector3<i64>, OctreeNode> = HashMap::new();
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        pending_nodes.push(root_node.clone());
        nodes.insert(root_node.get_center(), root_node);
        let mut targetted_node: Option<OctreeNode> = None;
        let mut closest_dist: f32 = f32::MAX;
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            // Check if this octree node is the targeted node
            let dist = veclib::Vector3::<f32>::from(octree_node.get_center()).distance(*target);
            if octree_node.depth == self.depth - 1 && dist < closest_dist {
                targetted_node = Some(octree_node.clone());
                closest_dist = dist;
            }
            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.depth) {
                pending_nodes.extend(octree_node.subdivide());
            }
            // Bruh
            pending_nodes.remove(0);
            nodes.insert(octree_node.get_center(), octree_node);
        }
        return (nodes, targetted_node);
    }
    // Use an already existing octree to generate new nodes based if each node passes the postprocess pass 
    pub fn generate_postprocess(&self, octree: &HashMap<veclib::Vector3<i64>, OctreeNode>, target: &veclib::Vector3<f32>) -> HashMap<veclib::Vector3<i64>, OctreeNode> {
        let mut nodes: HashMap<veclib::Vector3<i64>, OctreeNode> = HashMap::new();
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        // Add all the nodes that aren't the max-depth nodes
        pending_nodes.extend(octree.iter().filter_map(|(_, node)| { 
            if node.depth < self.depth - 1 {
                Some(node.clone())
            } else {
                None
            }
        }));
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            // If the node contains the position, subdivide it
            if octree_node.can_subdivide_postprocess(&target, self.lod_factor, self.depth) {
                pending_nodes.extend(octree_node.subdivide());
            }
            // Bruh
            nodes.insert(octree_node.get_center(), octree_node);
            pending_nodes.remove(0);
        }
        return nodes;
    }
    // Generate the base octree with a target point at 0, 0, 0
    pub fn generate_base_octree(&mut self) {
        let root_size = (2_u64.pow(self.depth as u32) * self.size as u64) as i64;
        let root_position = veclib::Vector3::<i64>::new(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        // Create the root node
        let root_node = OctreeNode {
            position: root_position,
            half_extent: (root_size / 2) as u64,
            depth: 0,
            parent_center: veclib::Vector3::<i64>::default_zero(),
            children_centers: [veclib::Vector3::<i64>::default_zero(); 8],
            children: false,
        };
        let octree_data = self.generate_octree(&veclib::Vector3::<f32>::default_one(), root_node);
        self.nodes = octree_data.0;
        self.targetted_node = octree_data.1;
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(&mut self, input: OctreeInput) {
        let instant = Instant::now();
        self.added_nodes.clear();
        self.removed_nodes.clear();
        // If we don't have a targetted node, exit early
        if self.targetted_node.is_none() {
            return;
        }
        println!("Step 1");    
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
        if marked_node.is_none() || node_to_remove.is_none() {
            return;
        }
        println!("Step 2");
        // Then we generate a local octree, using that marked node as the root
        let local_octree_data = self.generate_octree(&input.target, marked_node.clone().unwrap());
        self.targetted_node = local_octree_data.1;
        // Get the nodes that we've added
        let added_nodes = local_octree_data.0;

        // Set the added nodes
        self.added_nodes = added_nodes
            .values()
            .map(|x| x.clone())
            .filter(|x| {
                let valid_in_nodes = self.nodes.get(&x.get_center());
                match valid_in_nodes {
                    Some(_) => false,
                    None => true,
                }
            })
            .collect();

        // Add the delta to the nodes
        self.nodes.extend(added_nodes.clone());

        // Get the nodes that we've deleted
        let mut deleted_centers: HashSet<veclib::Vector3<i64>> = HashSet::new();
        {
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            pending_nodes.push(node_to_remove.clone().unwrap());
            // Recursively delete the nodes
            while pending_nodes.len() > 0 {
                let current_node = pending_nodes[0].clone();
                // Just in case
                if current_node.children {
                    // Get the children
                    for child_center in current_node.children_centers {
                        if child_center != veclib::Vector3::<i64>::default_zero() {
                            let child_node = self.nodes.get(&child_center).unwrap().clone();
                            pending_nodes.push(child_node);
                        }
                    }
                }
                deleted_centers.insert(current_node.get_center());
                pending_nodes.remove(0);
            }
        }

        // Update the removed node
        let mut node_to_remove = node_to_remove.unwrap();
        node_to_remove.children = false;
        node_to_remove.children_centers = [veclib::Vector3::<i64>::default_zero(); 8];
        self.added_nodes.push(node_to_remove.clone());
        self.nodes.insert(node_to_remove.get_center(), node_to_remove.clone());

        let center: veclib::Vector3<i64> = marked_node.as_ref().unwrap().get_center();
        let depth: u8 = marked_node.as_ref().unwrap().depth;
        println!("Time in micros: {}", instant.elapsed().as_micros());        
        // Remove the nodes
        // TODO: Optimize this
        self.removed_nodes = self
            .nodes
            .iter()
            .filter_map(|(&coord, node)| {
                // Check if this node should be removed
                let mut valid: bool = node.children || center == coord;
                if depth != 0 {
                    valid |= node.parent_center == center && node.can_subdivide(&input.target, self.depth);
                }
                if deleted_centers.contains(&coord) && coord != node_to_remove.get_center() || valid {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .collect();
        self.nodes.retain(|k, _| !deleted_centers.contains(k) || *k == node_to_remove.get_center());

        // Filter out the nodes that are already in the postprocess tree
        let mut nodes_clones = self.nodes.clone();
        let postprocess_nodes = self.generate_postprocess(&nodes_clones, &input.target);
        nodes_clones.extend(postprocess_nodes.clone());

        // The filtered newly added postprocessing nodes
        let new_postprocess_nodes = postprocess_nodes.iter().filter_map(|(coord, node)| {
            if self.nodes.contains_key(coord) {
                // This node already exists, that means that it didn't change
                None
            } else {
                // This node is a new node
                Some((*coord, node.clone()))
            }
        }).collect::<HashMap<veclib::Vector3<i64>, OctreeNode>>();
        
        self.final_nodes = nodes_clones;
    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,

    // Used for the parent-children links
    // TODO: Change this to it uses IDs instead of coordinates
    pub parent_center: veclib::Vector3<i64>,
    pub children_centers: [veclib::Vector3<i64>; 8],
    // Check if we had children
    pub children: bool,
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
        let aabb = self.get_aabb().min.elem_lte(target).all() && self.get_aabb().max.elem_gte(target).all();
        return aabb && self.depth < (max_depth - 1);
    }
    // Check if we can subdivide this node during the postprocessing loop
    pub fn can_subdivide_postprocess(&self, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let mut aabb = self.get_aabb();
        aabb.expand(lod_factor * self.half_extent as f32);
        return Intersection::point_aabb(target, &aabb) && self.depth < (max_depth - 1) && !self.children;
    }
    // Subdivide this node into 8 smaller nodes
    pub fn subdivide(&mut self) -> Vec<OctreeNode> {
        let extent_i64 = self.half_extent as i64;
        let mut output: Vec<OctreeNode> = Vec::new();
        let mut i: u16 = 0;
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    // The position offset for the new octree node
                    let offset: veclib::Vector3<i64> = veclib::Vector3::<i64>::new(x * extent_i64, y * extent_i64, z * extent_i64);
                    let child = OctreeNode {
                        position: self.position + offset,
                        half_extent: self.half_extent / 2,
                        depth: self.depth + 1,
                        parent_center: self.get_center(),
                        children_centers: [veclib::Vector3::<i64>::default_zero(); 8],
                        children: false,
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
