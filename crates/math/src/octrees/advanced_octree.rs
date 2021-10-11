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
    // Calculate the nodes that are the twin nodes *and* normal nodes
    // Twin nodes are basically just normal nodes that get subdivided after the main octree generation
    fn calculate_combined_nodes(&self, target: &veclib::Vector3<f32>, nodes: &Vec<OctreeNode>, lod_factor: f32) -> Vec<OctreeNode> {
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();
        
        // Output nodes
        let mut output_nodes: Vec<OctreeNode> = Vec::new();
        
        // Add all the normal nodes
        let other_iter = nodes.iter().map(|x| x.clone());
        pending_nodes.extend(other_iter);

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node passes the collision check, subdivide it
            if self.can_node_subdivide_twin(&octree_node, target) {
                let t = octree_node.subdivide();
                nodes[octree_node.index] = octree_node;
                pending_nodes.extend(t.clone());
                nodes.extend(t);
            }

            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
            output_nodes.push(octree_node);
        }

        return output_nodes;
    }    
    // Generate the base octree with a target point at 0, 0, 0
    fn generate_base_octree(&mut self, lod_factor: f32) -> Vec<OctreeNode> {
        // Create the root node
        let root_node = self.octree.get_root_node();
        self.octree.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        self.calculate_combined_nodes(&veclib::Vector3::ONE, &self.octree.nodes, lod_factor)
    }
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(
        &mut self,
        target: &veclib::Vector3<f32>,
        lod_factor: f32,
    ) -> Option<(
        Vec<OctreeNode>, // Added nodes
        Vec<OctreeNode>, // Removed nodes
        Vec<OctreeNode>, // Total nodes
    )> {
        // Clamp the input position
        let root_node = self.octree.get_root_node();
        let input: veclib::Vector3<f32> = veclib::Vector3::<f32>::clamp(
            *target,
            veclib::Vector3::<f32>::from(root_node.position) + 32.0,
            veclib::Vector3::<f32>::from(root_node.position + (root_node.half_extent * 2) as i64) - 32.0,
        );
        // Check if we even have the base octree generated
        if !self.generated_base_octree {
            // The base octree is not generated, so generate it
            let added_nodes = self.generate_base_octree(lod_factor);
            return Some((added_nodes, Vec::new(), added_nodes));
        }
        // If we don't have a target node don't do anything
        if self.octree.target_node.is_none() {
            return None;
        }

        // What we do for incremental generation
        // We go up the tree from the target node, then we check the highest depth node that still has a collision with the target point
        // From there, we go down the tree and generate a sub-octree, then we just append it to our normal octree
        // The highest depth node with that contains the target point
        
        // Loop through the tree recursively
        let mut current_node = self.octree.target_node.unwrap();
        let common_target_node: OctreeNode; 
        while current_node.depth != self.octree.depth {
            // Go up the tree
            let parent = self.octree.nodes.get_element(current_node.parent_index).unwrap();
            // Check for collisions
            if parent.can_subdivide(target, self.octree.depth) {
                // This node the common target node
                common_target_node = parent.clone();
                break;
            }
        }
        
        // Check if we even changed parents
        let new_parents = self.octree.target_node.unwrap().parent_index != common_target_node.index;
        if new_parents {
            // Just gotta do this I guess

            
        }  
        return None;
    }
}