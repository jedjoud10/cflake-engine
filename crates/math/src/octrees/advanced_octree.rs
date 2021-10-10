use std::collections::HashMap;
use std::collections::HashSet;

// The whole octree
pub struct AdvancedOctree {
    
    pub postprocess_nodes: HashMap<veclib::Vector3<i64>, OctreeNode>,
    pub size: u64,
    pub generated_base_octree: bool,
}

impl Default for AdvancedOctree {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            postprocess_nodes: HashMap::new(),
            targetted_node: None,
            size: 1,
            depth: 1,
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
            path: Vec::new(),
        }
    }
    // Get the subdivided nodes that have passed through the post process check
    pub fn calculate_postprocess_nodes(&self, target: &veclib::Vector3<f32>, nodes: &HashMap<veclib::Vector3<i64>, OctreeNode>, lod_factor: f32) -> HashMap<veclib::Vector3<i64>, OctreeNode> {
        let mut output: HashMap<veclib::Vector3<i64>, OctreeNode> = HashMap::new();
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        pending_nodes.extend(nodes.iter().map(|x| x.1.clone()));
        while pending_nodes.len() > 0 {
            let mut octree_node = pending_nodes[0].clone();
            // If the node passes the postprocess check, subdivide it, though only if it has no children
            if octree_node.can_subdivide_postprocess(target, lod_factor, self.depth) && !octree_node.children {
                let t = octree_node.subdivide();
                pending_nodes.extend(t);
            }
            // Bruh
            pending_nodes.remove(0);
            output.insert(octree_node.get_center(), octree_node);
        }
        return output;
    }    
    // Generate the base octree with a target point at 0, 0, 0
    fn generate_base_octree(&mut self, lod_factor: f32) -> HashMap<veclib::Vector3<i64>, OctreeNode> {
        // Create the root node
        let root_node = self.get_root_node();
        let octree_data = self.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        self.nodes = octree_data.0.clone();
        self.targetted_node = octree_data.1;
        let mut nodes = octree_data.0;
        let postprocess_nodes = self.calculate_postprocess_nodes(&veclib::Vector3::ONE, &nodes, lod_factor);
        self.postprocess_nodes = postprocess_nodes.clone();
        nodes.extend(postprocess_nodes);
        return nodes;
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(
        &mut self,
        input: veclib::Vector3<f32>,
        lod_factor: f32,
    ) -> Option<(
        HashMap<veclib::Vector3<i64>, OctreeNode>,
        HashMap<veclib::Vector3<i64>, OctreeNode>,
        HashMap<veclib::Vector3<i64>, OctreeNode>,
    )> {
        // Clamp the input position
        let input: veclib::Vector3<f32> = veclib::Vector3::<f32>::clamp(
            input,
            veclib::Vector3::<f32>::from(self.get_root_node().position) + 32.0,
            veclib::Vector3::<f32>::from(self.get_root_node().position + (self.get_root_node().half_extent * 2) as i64) - 32.0,
        );
        // Check if we even have the base octree generated
        if !self.generated_base_octree {
            // The base octree is not generated, so generate it
            let added_nodes = self.generate_base_octree(lod_factor);
            let nodes = added_nodes.clone();
            self.generated_base_octree = true;
            return Some((added_nodes, HashMap::new(), nodes));
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
        if !marked_node.is_none() && !node_to_remove.is_none() {
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
        }       

        // Subdivide each added node at least once
        let postprocess_nodes: HashMap<veclib::Vector3<i64>, OctreeNode> = self.calculate_postprocess_nodes(&input, &self.nodes, lod_factor);
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
        let added_nodes_hashmap = added_postprocess_nodes
            .iter()
            .map(|x| (x.get_center(), x.clone()))
            .collect::<HashMap<veclib::Vector3<i64>, OctreeNode>>();
        let removed_nodes_hashmap: HashMap<veclib::Vector3<i64>, OctreeNode> = removed_postprocess_nodes.iter().map(|x| (x.get_center(), x.clone())).collect();
        // Return
        return Some((added_nodes_hashmap, removed_nodes_hashmap, self.postprocess_nodes.clone()));
    }
}