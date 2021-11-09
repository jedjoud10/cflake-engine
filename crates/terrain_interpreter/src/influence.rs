use math::constructive_solid_geometry::{CSGShape, CSGTree};
use crate::var_hash_getter::VarHashGetter;
// An influence struct that will be passed upon node generations
#[derive(Clone, Debug)]
pub struct Influence {
    pub csgtree: CSGTree
}

impl Influence {
    // Expand a specific node
    // Range is a positive number that will expand the specified node outwards in one direction
    pub fn expand(&mut self, node_index: usize, range: f32) {
        let t = self.csgtree.get_mut(node_index);
        t.expand(math::csg::ExpandMethod::Factor(range));
    }
    // Add a specific node
    pub fn add(&mut self, node: CSGShape) {
        self.csgtree.add(node);
    }
}