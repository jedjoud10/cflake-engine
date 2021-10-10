use super::{node::OctreeNode, octree::Octree};

// An advanced octree with incremental generation and twin nodes
pub struct AdvancedOctree {    
    // The original octree
    pub octree: Octree,
    pub generated_base_octree: bool,
}

impl AdvancedOctree {   
    // Check if a node can be subdivided
    fn can_node_subdivide_twin(&self, node: &OctreeNode, target: &veclib::Vector3<f32>) -> bool {
        // Only subdivide if we don't have any children
        node.children_indices.is_none();
        false
    }
    // Calculate the twin nodes
    // Twin nodes are basically just normal nodes that get subdivided after the main octree generation
    fn get_twin_combined_nodes(&self, target: &veclib::Vector3<f32>, nodes: &Vec<OctreeNode>, lod_factor: f32) -> Vec<OctreeNode> {
        // The output twin nodes
        let mut twin_nodes: Vec<Option<OctreeNode>> = Vec::new();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        
        // Add all the normal nodes
        let other_iter = nodes.iter().filter_map(|x| x.as_ref().cloned());
        pending_nodes.extend(other_iter);

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node passes the collision check, subdivide it
            if self.can_node_subdivide_twin(&octree_node, target) {
                let t = octree_node.subdivide();
                pending_nodes.extend(t);
            }

            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
            twin_nodes.push(Some(octree_node));
        }
        return twin_nodes;
    }    
    // Generate the base octree with a target point at 0, 0, 0
    fn generate_base_octree(&mut self, lod_factor: f32) {
        // Create the root node
        let root_node = self.octree.get_root_node();
        let (normal_nodes, targetted_node) = self.octree.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        let mut nodes = normal_nodes.0;
        let twin_nodes = self.get_twin_nodes(&veclib::Vector3::ONE, &nodes, lod_factor);
        self.twin_nodes = twin_nodes.clone();
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(
        &mut self,
        input: veclib::Vector3<f32>,
        lod_factor: f32,
    ) -> Option<(
        Vec<OctreeNode>, // Added nodes
        Vec<OctreeNode>, // Removed nodes
        Vec<OctreeNode>, // Total nodes
    )> {
        // Clamp the input position
        let root_node = self.octree.get_root_node();
        let input: veclib::Vector3<f32> = veclib::Vector3::<f32>::clamp(
            input,
            veclib::Vector3::<f32>::from(root_node.position) + 32.0,
            veclib::Vector3::<f32>::from(root_node.position + (root_node.half_extent * 2) as i64) - 32.0,
        );
        // Check if we even have the base octree generated
        if !self.generated_base_octree {
            // The base octree is not generated, so generate it
            self.generate_base_octree(lod_factor);
            // Combine the normal nodes and the twin nodes together
            let combined_nodes = 
            return Some((self., Vec::new(), nodes));
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