use super::{node::OctreeNode, octree::Octree};
use ordered_vec::simple::UnversionnedOrderedVec;
use std::collections::{HashMap, HashSet};

// An advanced octree with incremental generation and twin nodes
#[derive(Default)]
pub struct AdvancedOctree {
    // The original octree
    pub internal_octree: Octree,
    // Did we generate the base octree already?
    pub generated_base_octree: bool,
    // The combined twin and normal nodes
    pub combined_nodes: HashSet<OctreeNode>,
    // The last position
    pub position: veclib::Vector3<f32>,
    // The twin rule, if this is none, don't generate twin nodes
    pub subdivide_twin_rule: Option<fn(&OctreeNode, &veclib::Vector3<f32>, f32, u8) -> bool>,
}

// Code
// TODO: Multithread this and completely rewrite it. Remember: K.I.S.S principle
impl AdvancedOctree {
    // New
    pub fn new(octree: Octree, subdivide_twin_rule: fn(&OctreeNode, &veclib::Vector3<f32>, f32, u8) -> bool) -> Self {
        Self {
            internal_octree: octree,
            subdivide_twin_rule: Some(subdivide_twin_rule),
            ..AdvancedOctree::default()
        }
    }
    // Twin nodes are basically just normal nodes that get subdivided after the main octree generation
    fn calculate_combined_nodes(
        twin_rule: fn(&OctreeNode, &veclib::Vector3<f32>, f32, u8) -> bool,
        target: &veclib::Vector3<f32>,
        nodes: &UnversionnedOrderedVec<OctreeNode>,
        lod_factor: f32,
        max_depth: u8,
    ) -> HashSet<OctreeNode> {
        let mut combined_nodes: UnversionnedOrderedVec<OctreeNode> = nodes.clone();
        // The nodes that must be evaluated
        let mut pending_nodes: Vec<OctreeNode> = nodes.iter_elements().cloned().collect();
        // Evaluate each node
        while !pending_nodes.is_empty() {
            // Get the current pending node
            let mut octree_node = pending_nodes[0].clone();

            // If the node passes the collision check, subdivide it
            if (twin_rule)(&octree_node, target, lod_factor, max_depth) {
                // Add the children nodes
                let child_nodes = octree_node.subdivide(&mut combined_nodes);
                pending_nodes.extend(child_nodes);
            }

            // Remove the node so we don't cause an infinite loop
            pending_nodes.remove(0);
        }
        return combined_nodes.iter_elements().cloned().collect::<HashSet<OctreeNode>>();
    }
}
// Base / incremental generation
impl AdvancedOctree {
    // Generate the octree at a specific position with a specific depth
    pub fn generate_incremental_octree(
        &mut self,
        target: &veclib::Vector3<f32>,
        lod_factor: f32,
    ) -> Option<(
        Vec<OctreeNode>, // Added nodes
        Vec<OctreeNode>, // Removed nodes
    )> {
        let root_node = self.internal_octree.get_root_node();
        let t = std::time::Instant::now();
        // Do nothing if the target is out of bounds
        if !crate::intersection::Intersection::point_aabb(target, &root_node.get_aabb()) {
            return None;
        }
        // Check if we generated the base octree
        if !self.generated_base_octree {
            // Create the root node
            let root_node = self.internal_octree.get_root_node();
            self.internal_octree.generate_octree(&veclib::Vector3::ONE, root_node);
            println!("Took '{}' micros to generate base octree", t.elapsed().as_micros());
            let mut added_nodes: Vec<OctreeNode> = self.internal_octree.nodes.iter_elements().cloned().collect();
            self.generated_base_octree = true;
            match self.subdivide_twin_rule {
                Some(x) => {
                    // Further subdivision
                    let y = Self::calculate_combined_nodes(x, target, &self.internal_octree.nodes, lod_factor, self.internal_octree.depth);
                    added_nodes = y.clone().into_iter().collect();
                    self.combined_nodes = y;
                    return Some((added_nodes, Vec::new()));
                }
                None => {
                    self.combined_nodes = added_nodes.iter().cloned().collect();
                    return Some((added_nodes, Vec::new()));
                }
            }
        }
        // If we don't have a target node don't do anything
        self.internal_octree.target_node.as_ref()?;
        // If the target has not moved, we don't need to incrementally update the octree
        if self.position.distance(*target) < 1.0 {
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

        while current_node.depth != 0 {
            // Go up the tree
            let parent = self.internal_octree.nodes.get(current_node.parent_index).unwrap();
            // Check for collisions
            if parent.can_subdivide(target, self.internal_octree.depth) || parent.depth == 0 {
                // This node the common target node
                common_target_node = parent.clone();
                break;
            }
            // Update the current node
            current_node = parent.clone();
        }

        // Recursively get the removed nodes from the common target's node first valid child
        let removed_nodes: HashMap<veclib::Vector3<i64>, usize> = common_target_node
            .find_children_recursive(&self.internal_octree.nodes)
            .iter()
            .map(|x| (x.get_center(), x.index))
            .collect();
        let different_parent = self.internal_octree.target_node.as_ref().unwrap().parent_index != common_target_node.index;
        if different_parent {
            // Update the children
            let a = self.internal_octree.nodes.get_mut(common_target_node.index).unwrap();
            a.children_indices = None;

            // ----Update the normal nodes first, just normal sub-octree generation. We detect added/removed nodes at the end----
            // Final nodes
            let mut nodes: UnversionnedOrderedVec<OctreeNode> = self.internal_octree.nodes.clone();
            // The nodes that must be evaluated
            let mut pending_nodes: Vec<OctreeNode> = Vec::new();
            // The targetted node that is specified using the target position
            let mut target_node: Option<OctreeNode> = None;
            pending_nodes.push(common_target_node);
            // Evaluate each node
            while !pending_nodes.is_empty() {
                // Get the current pending node
                let mut octree_node = pending_nodes[0].clone();

                // Update target node
                if octree_node.depth == self.internal_octree.depth - 1 && octree_node.can_subdivide(target, self.internal_octree.depth + 1) {
                    target_node = Some(octree_node.clone());
                }

                // If the node contains the position, subdivide it
                if octree_node.can_subdivide(target, self.internal_octree.depth) {
                    // Add each child node, but also update the parent's child link id
                    let child_nodes = octree_node.subdivide(&mut nodes);
                    pending_nodes.extend(child_nodes.clone());
                }

                // So we don't cause an infinite loop
                pending_nodes.remove(0);
            }
            // Compensate for the removed nodes
            for (_, index) in removed_nodes.iter() {
                nodes.remove(*index).unwrap();
            }
            self.internal_octree.extern_update(target_node, nodes.clone());
        }
        // Should we generate twin nodes?
        let new_hashset = match self.subdivide_twin_rule {
            Some(twin_rule) => {
                // Yep, generate the twin nodes
                Self::calculate_combined_nodes(twin_rule, target, &self.internal_octree.nodes, lod_factor, self.internal_octree.depth)
            }
            None => {
                // Nope, just take the newly generated nodes and get the diff
                self.internal_octree.nodes.iter_elements().cloned().collect::<HashSet<OctreeNode>>()
            }
        };
        // The old hashset
        let old_hashset = &self.combined_nodes;

        // Now actually detect the removed / added nodes
        let removed_nodes = old_hashset.difference(&new_hashset).cloned().collect();
        let added_nodes = new_hashset.difference(old_hashset).cloned().collect();
        self.combined_nodes = new_hashset;
        self.position = *target;
        Some((added_nodes, removed_nodes))
    }
}
