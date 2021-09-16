use super::intersection::Intersection;
use super::shapes;
use super::shapes::Sphere;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::{BitAnd, BitOr};
use std::time::Instant;

// TODO: Rewrite this
// The whole octree
pub struct Octree {
    pub nodes: HashMap<veclib::Vector3<i64>, OctreeNode>,
    pub targetted_node: Option<OctreeNode>,
    pub postprocess_nodes: HashMap<veclib::Vector3<i64>, OctreeNode>,
    pub lod_factor: f32,
    pub size: u64,
    pub depth: u8,
    pub generated_base_octree: bool,
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            postprocess_nodes: HashMap::new(),
            targetted_node: None,
            size: 1,
            depth: 1,
            lod_factor: 1.0,
            generated_base_octree: false,
        }
    }
}

// TODO: Rewrite this
impl Octree {
    // Get the root node of this octree
    pub fn get_root_node(&self) -> OctreeNode {
        let root_size = (2_u64.pow(self.depth as u32) * self.size as u64) as i64;
        let root_position = veclib::Vector3::<i64>::new(-(root_size / 2), -(root_size / 2), -(root_size / 2));
        OctreeNode {
            position: root_position,
            half_extent: (root_size / 2) as u64,
            depth: 0,
            postprocess: false,
            parent_center: veclib::Vector3::<i64>::ZERO,
            children_centers: [veclib::Vector3::<i64>::ZERO; 8],
            child_leaf_count: 0,
            children: false,
        }
    }
    // Get the subdivided nodes that have passed through the post process check
    pub fn calculate_postprocess_nodes(&self, target: &veclib::Vector3<f32>, nodes: &HashMap<veclib::Vector3<i64>, OctreeNode>) -> HashMap<veclib::Vector3<i64>, OctreeNode> {
        let mut output: HashMap<veclib::Vector3<i64>, OctreeNode> = HashMap::new();
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        pending_nodes.extend(nodes.iter().map(|x| x.1.clone()));
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            // If the node passes the postprocess check, subdivide it, though only if it has no children
            if octree_node.can_subdivide_postprocess(target, self.lod_factor, self.depth) && !octree_node.children {
                let t = octree_node.subdivide();
                pending_nodes.extend(t);
            }
            // Bruh
            pending_nodes.remove(0);
            output.insert(octree_node.get_center(), octree_node);
        }
        return output;
    }
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
    // Generate the base octree with a target point at 0, 0, 0
    pub fn generate_base_octree(&mut self) -> HashMap<veclib::Vector3<i64>, OctreeNode> {
        // Create the root node
        let root_node = self.get_root_node();
        let octree_data = self.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        self.nodes = octree_data.0.clone();
        self.targetted_node = octree_data.1;
        let mut nodes = octree_data.0;
        let postprocess_nodes = self.calculate_postprocess_nodes(&veclib::Vector3::ONE, &nodes);
        self.postprocess_nodes = postprocess_nodes.clone();
        nodes.extend(postprocess_nodes);
        println!("Generated the base octree");
        return nodes;
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(&mut self, input: veclib::Vector3<f32>) -> Option<(Vec<OctreeNode>, Vec<OctreeNode>, HashMap<veclib::Vector3::<i64>, OctreeNode>)> {
        // Clamp the input position
        let input: veclib::Vector3<f32> = veclib::Vector3::<f32>::clamp(
            input,
            veclib::Vector3::<f32>::from(self.get_root_node().position) + 32.0,
            veclib::Vector3::<f32>::from(self.get_root_node().position + (self.get_root_node().half_extent * 2) as i64) - 32.0,
        );
        // Check if we even have the base octree generated
        if !self.generated_base_octree {
            // The base octree is not generated, so generate it
            let added_nodes = self.generate_base_octree();
            let nodes = added_nodes.clone();
            let added_nodes: Vec<OctreeNode> = added_nodes.iter().map(|(center, node)| node.clone()).collect();
            self.generated_base_octree = true;
            return Some((added_nodes, Vec::new(), nodes));
        }
        // If we don't have a targetted node try to create the base octree
        if self.targetted_node.is_none() {
            return None;
        }
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
                intersection = current_node.can_subdivide(&input, self.depth);
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
            return None;
        }
        // Then we generate a local octree, using that marked node as the root
        let local_octree_data = self.generate_octree(&input, marked_node.clone().unwrap());
        self.targetted_node = local_octree_data.1;
        // Get the nodes that we've added
        let added_nodes = local_octree_data.0;
        // Update the actual nodes before we calculate the postprocessed nodes
        self.nodes.extend(added_nodes.clone());

        // Get the nodes that we've deleted
        let mut deleted_centers: HashSet<veclib::Vector3<i64>> = HashSet::new();
        {
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            pending_nodes.push(node_to_remove.clone().unwrap());
            // Recursively delete the nodes
            while pending_nodes.len() > 0 {
                let current_node = pending_nodes[0].clone();
                // Recursively remove the nodes
                if current_node.children {
                    // Get the children
                    for child_center in current_node.children_centers {
                        // Just in case
                        if child_center != veclib::Vector3::<i64>::ZERO {
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
        node_to_remove.children_centers = [veclib::Vector3::<i64>::ZERO; 8];
        self.nodes.insert(node_to_remove.get_center(), node_to_remove.clone());
        self.nodes.retain(|k, _| !deleted_centers.contains(k) || *k == node_to_remove.get_center());

        // Subdivide each added node at least once
        let postprocess_nodes: HashMap<veclib::Vector3<i64>, OctreeNode> = self.calculate_postprocess_nodes(&input, &self.nodes);
        let mut removed_postprocess_nodes: Vec<OctreeNode> = Vec::new();
        // Detect the newly made postprocess-nodes
        let mut added_postprocess_nodes: Vec<OctreeNode> = Vec::new();
        for (k, node) in postprocess_nodes.iter() {
            if !self.postprocess_nodes.contains_key(k) {
                // We added the node
                added_postprocess_nodes.push(node.clone());
            } else {
                // We didn't change the node / it changed it's children status
                if !node.children && self.postprocess_nodes[k].children {
                    // We don't have children anymore, so this node counts as a new node
                    added_postprocess_nodes.push(node.clone());
                }
            }
        }
        // Detect the removed nodes
        for (k, node) in self.postprocess_nodes.iter() {
            if !postprocess_nodes.contains_key(k) {
                // We removed the node
                removed_postprocess_nodes.push(node.clone());
            } else {
                // We didn't change the node / it changed it's children status
                if !node.children && postprocess_nodes[k].children {
                    // We have children now, so this counts as a removed node
                    removed_postprocess_nodes.push(node.clone());
                }
            }
        }

        // Update
        self.postprocess_nodes = postprocess_nodes;   
        let added_nodes_hashmap = added_postprocess_nodes.iter().map(|x| (x.get_center(), x.clone())).collect::<HashMap<veclib::Vector3<i64>, OctreeNode>>();
        // Return
        return Some((added_postprocess_nodes, removed_postprocess_nodes, added_nodes_hashmap));
    }
}

// Simple node in the octree
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub position: veclib::Vector3<i64>,
    pub half_extent: u64,
    pub depth: u8,
    pub child_leaf_count: u8,

    // Used for the parent-children links
    // TODO: Change this to it uses IDs instead of coordinates
    pub parent_center: veclib::Vector3<i64>,
    pub children_centers: [veclib::Vector3<i64>; 8],
    // Check if we passed the postprocess test
    pub postprocess: bool,
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
        let aabb = self.get_aabb().min.elem_lte(target) & self.get_aabb().max.elem_gt(target);
        let aabb = (aabb | veclib::Vector3::<bool>::new(false, false, false)).all();
        return aabb && self.depth < (max_depth - 1);
    }
    // Check if we can subdivide this node during the postprocessing loop
    pub fn can_subdivide_postprocess(&self, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let mut aabb = self.get_aabb();
        aabb.expand(lod_factor * self.half_extent as f32);
        let aabb = aabb.min.elem_lte(target) & aabb.max.elem_gt(target);
        let aabb = (aabb | veclib::Vector3::<bool>::new(false, false, false)).all();
        return aabb && self.depth < (max_depth - 1);
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
                        postprocess: false,
                        children_centers: [veclib::Vector3::<i64>::ZERO; 8],
                        child_leaf_count: 0,
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
