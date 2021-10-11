use others::SmartList;

use super::{
    node::OctreeNode,
    octree::{self, Octree},
};

// An advanced octree with incremental generation and twin nodes
#[derive(Default)]
pub struct AdvancedOctree {
    // The original octree
    pub internal_octree: Octree,
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
    fn calculate_combined_nodes(&self, target: &veclib::Vector3<f32>, nodes: &mut SmartList<OctreeNode>, lod_factor: f32) {
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();

        // Start at the root of the tree
        pending_nodes.push(nodes.get_element(0).cloned().unwrap());

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node passes the collision check, subdivide it
            if self.can_node_subdivide_twin(&octree_node, target) {
                // Add the children nodes
                let child_nodes = octree_node.subdivide();
                pending_nodes.extend(child_nodes.clone());
                for child in child_nodes {
                    nodes.add_element(child);
                }
            }

            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
        }
    }
    // Generate the base octree with a target point at 0, 0, 0
    fn generate_base_octree(&mut self, lod_factor: f32) -> Vec<OctreeNode> {
        // Create the root node
        let root_node = self.internal_octree.get_root_node();
        self.internal_octree.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        //self.calculate_combined_nodes(&veclib::Vector3::ONE, &self.octree.nodes, lod_factor)
        return self.internal_octree.nodes.elements.iter().filter_map(|x| x.as_ref().cloned()).collect();
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
        let root_node = self.internal_octree.get_root_node();
        let input: veclib::Vector3<f32> = veclib::Vector3::<f32>::clamp(
            *target,
            veclib::Vector3::<f32>::from(root_node.position) + 32.0,
            veclib::Vector3::<f32>::from(root_node.position + (root_node.half_extent * 2) as i64) - 32.0,
        );
        // Check if we even have the base octree generated
        if !self.generated_base_octree {
            // The base octree is not generated, so generate it
            let added_nodes = self.generate_base_octree(lod_factor);
            return Some((added_nodes.clone(), Vec::new(), added_nodes));
        }
        // If we don't have a target node don't do anything
        if self.internal_octree.target_node.is_none() {
            return None;
        }

        // What we do for incremental generation
        // We go up the tree from the target node, then we check the highest depth node that still has a collision with the target point
        // From there, we go down the tree and generate a sub-octree, then we just append it to our normal octree
        // The highest depth node with that contains the target point

        // Loop through the tree recursively
        let mut current_node = self.internal_octree.target_node.as_ref().cloned().unwrap();
        // The deepest node that has a collision with the new target point
        let mut common_target_node: OctreeNode = self.internal_octree.target_node.as_ref().cloned().unwrap();
        while current_node.depth != self.internal_octree.depth {
            // Go up the tree
            let parent = self.internal_octree.nodes.get_element(current_node.parent_index).unwrap();
            // Check for collisions
            if parent.can_subdivide(target, self.internal_octree.depth) {
                // This node the common target node
                common_target_node = parent.clone();
                break;
            }
            // Update the current node
            current_node = parent.clone();
        }

        // Check if we even changed parents
        let target_node_index = self.internal_octree.target_node.as_ref().cloned().unwrap().parent_index;
        let new_parents = target_node_index != common_target_node.index;
        if new_parents {
            // ----Update the normal nodes first, just normal sub-octree generation. We detect added/removed nodes at the end----
            // Final nodes
            let mut nodes: SmartList<OctreeNode> = self.internal_octree.nodes.clone();
            // The nodes that must be evaluated
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            // The starting node
            pending_nodes.push(common_target_node);
            // The depth of the octree
            let depth = self.internal_octree.depth;

            // The targetted node that is specified using the target position
            let mut targetted_node: Option<OctreeNode> = None;

            // Evaluate each node
            while pending_nodes.len() > 0 {
                // Get the current pending node
                let mut octree_node = pending_nodes[0].clone();

                // Update target node
                if octree_node.depth == depth - 1 && octree_node.can_subdivide(target, depth + 1) {
                    targetted_node = Some(octree_node.clone());
                }

                // If the node contains the position, subdivide it
                if octree_node.can_subdivide(&target, depth) {
                    // Update the parent node
                    let elm = nodes.get_element_mut(octree_node.index).unwrap();
                    // Update the values
                    elm.children_indices = octree_node.children_indices;

                    // Add each child node, but also update the parent's child link id
                    let child_nodes = octree_node.subdivide();
                    pending_nodes.extend(child_nodes.clone());
                    for child in child_nodes {
                        nodes.add_element(child);
                    }
                }
            }
        }
        // Output
        return None;
    }
}
