use std::collections::{HashMap, HashSet};
use std::time::Instant;
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
    // The combined twin and normal nodes
    pub combined_nodes: SmartList<OctreeNode>
}

// Twin node generation
impl AdvancedOctree {
    // Check if a node can be subdivided
    fn can_node_subdivide_twin(&self, node: &OctreeNode, target: &veclib::Vector3<f32>) -> bool {
        // Only subdivide if we don't have any children
        node.children_indices.is_none();
        false
    }
    // Calculate the nodes that are the twin nodes *and* normal nodes
    // Twin nodes are basically just normal nodes that get subdivided after the main octree generation
    fn calculate_combined_nodes(&mut self, target: &veclib::Vector3<f32>, nodes: &SmartList<OctreeNode>, lod_factor: f32) {
        let mut combined_nodes: SmartList<OctreeNode> = nodes.clone();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();

        // Start at the root of the tree
        pending_nodes.push(nodes.get_element(0).unwrap().cloned().unwrap());

        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node passes the collision check, subdivide it
            if self.can_node_subdivide_twin(&octree_node, target) {
                // Add the children nodes
                let child_nodes = octree_node.subdivide(&mut combined_nodes);
            }

            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
        }

        self.combined_nodes = combined_nodes;
    }
}
// Base / incremental generation
impl AdvancedOctree {    
    // Generate the base octree with a target point at 0, 0, 0
    pub fn generate_base_octree(&mut self, lod_factor: f32) -> Vec<OctreeNode> {
        let t = std::time::Instant::now();
        // Create the root node
        let root_node = self.internal_octree.get_root_node();
        self.internal_octree.generate_octree(&veclib::Vector3::ONE, root_node.clone());
        //self.calculate_combined_nodes(&veclib::Vector3::ONE, &self.octree.nodes, lod_factor)
        println!("Took '{}' micros to generate base octree", t.elapsed().as_micros());
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
        let root_node = self.internal_octree.get_root_node();
        // Do nothing if the target is out of bounds
        if !crate::intersection::Intersection::point_aabb(target, &root_node.get_aabb()) {
            return None;
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
        /*
        for (i, node) in self.internal_octree.nodes.elements.iter().enumerate() {
            match node {
                Some(x) => println!("I: {}, N: {:?}", i, x),
                None => {},
            }
        }
        */
        while current_node.depth != 0 {
            // Go up the tree
            let parent = self.internal_octree.nodes.get_element(current_node.parent_index).unwrap().unwrap();
            // Check for collisions
            if parent.can_subdivide(&target, self.internal_octree.depth) || parent.depth == 0 {
                // This node the common target node
                common_target_node = parent.clone();
                break;
            }      
            // Update the current node
            current_node = parent.clone();
        }

        // Recursively get the removed nodes from the common target's node first valid child
        let removed_nodes: HashMap<veclib::Vector3<i64>, usize> = common_target_node.find_children_recursive(&self.internal_octree.nodes).iter().map(|x| (x.get_center(), x.index)).collect();
        // Early quit tests
        let new_parent = self.internal_octree.target_node.as_ref().unwrap().parent_index == common_target_node.index;
        // It is currently 10:41 on a wednesday night. I want to die
        if new_parent {
            return None;
        }

        // Update the children
        let a = self.internal_octree.nodes.get_element_mut(common_target_node.index).unwrap().unwrap();
        a.children_indices = None;

        // ----Update the normal nodes first, just normal sub-octree generation. We detect added/removed nodes at the end----
        // Keep track of the starting hashset
        let old_hashset = self
            .internal_octree
            .nodes
            .elements
            .iter()
            .filter_map(|x| match x {
                Some(x) => Some(x.clone()),
                None => None,
            }).collect::<HashSet<OctreeNode>>();
            
        // Final nodes
        let mut nodes: SmartList<OctreeNode> = self.internal_octree.nodes.clone();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = Vec::new();        
        // The targetted node that is specified using the target position
        let mut target_node: Option<OctreeNode> = None;
        pending_nodes.push(common_target_node);
        // Evaluate each node
        while pending_nodes.len() > 0 {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // Update target node
            if octree_node.depth == self.internal_octree.depth - 1 && octree_node.can_subdivide(&target, self.internal_octree.depth + 1) {
                target_node = Some(octree_node.clone());
            }

            // If the node contains the position, subdivide it
            if octree_node.can_subdivide(&target, self.internal_octree.depth) {
                // Add each child node, but also update the parent's child link id
                let child_nodes = octree_node.subdivide(&mut nodes);
                pending_nodes.extend(child_nodes.clone());
            }

            // So we don't cause an infinite loop
            pending_nodes.remove(0);
        }
        // Just some basic remapping of the values
        let removed_nodes = removed_nodes
            .iter()
            .map(|(_, index)| nodes.get_element(*index).unwrap().unwrap().clone())
            .collect::<Vec<OctreeNode>>();
        // Compensate for the removed nodes     
        for node in removed_nodes.iter() {
            nodes.remove_element(node.index).unwrap();
        }        
        self.internal_octree.extern_update(target_node, nodes.clone());  
        // The new hashset
        let new_hashset = nodes.elements.iter().filter_map(|x| x.as_ref().cloned()).collect::<HashSet<OctreeNode>>();
        // Now actually detect the removed / added nodes
        let removed_nodes = old_hashset.difference(&new_hashset).cloned().collect();
        let added_nodes = new_hashset.difference(&old_hashset).cloned().collect();

        let nodes = nodes.elements.iter().filter_map(|x| match x.as_ref() {
            Some(x) => Some(x.clone()),
            None => None,
        }).collect::<Vec<OctreeNode>>();
        return Some((added_nodes, removed_nodes, nodes));        
    }
}
