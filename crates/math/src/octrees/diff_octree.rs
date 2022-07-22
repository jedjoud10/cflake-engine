use ahash::AHashSet;
use super::{HeuristicSettings, Node, Octree};

// A differential octree, so we can detect what nodes we have added/removes from this octree
#[derive(Default)]
pub struct DiffOctree {
    // Underlying simple octree
    inner: Octree,

    // A set containing all the nodes from the previous update
    previous: AHashSet<Node>,
}

impl DiffOctree {
    // Create a new octree with a specific depth
    pub fn new(depth: u8, size: u64, hsettings: HeuristicSettings) -> Self {
        Self {
            inner: Octree::new(depth, size, hsettings),
            previous: Default::default(),
        }
    }
    
    // Update the differential, and return the values of new added nodes and old removed nodes
    pub fn update(&mut self, target: vek::Vec3<f32>) -> Option<(Vec<Node>, Vec<Node>)> {
        // Keep track of the previous nodes
        let success = self.inner.update(target);
        let result = if success.is_some() {
            // We successfully updated the simple octree, so we must check differences now
            let current = self
                .inner
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<AHashSet<_>>();

            // And check for differences
            let removed = self
                .previous
                .difference(&current)
                .cloned()
                .collect::<Vec<_>>();
            let added = current
                .difference(&self.previous)
                .cloned()
                .collect::<Vec<_>>();
            self.previous = current;
            Some((added, removed))
        } else {
            None
        };
        result
    }
}
