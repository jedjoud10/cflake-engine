use std::collections::{HashMap, HashSet};

use math::constructive_solid_geometry::CSGTree;

use crate::{
    error::InterpreterError,
    nodes::*,
    var_hash::{PassedData, VarHash, VarHashType},
    var_hash_getter::VarHashGetter,
    Node, NodeInterpreter,
};

// The main system that will be made from multiple densities and combiners
pub struct Interpreter {
    pub nodes: Vec<Node>,
    pub used_nodes: Vec<usize>,
    pub vars: Vec<VarHash>,
    pub lines: Vec<String>,
    pub finalized: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        // Create the default starter node
        Self {
            nodes: Vec::new(),
            used_nodes: Vec::new(),
            vars: Vec::new(),
            lines: Vec::new(),
            finalized: false,
        }
    }
}

// Add nodes
impl Interpreter {
    // Default pre-generated
    pub fn new_pregenerated() -> Self {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();
        // Add the default pos.y nodes
        let p = BasePosition.new(&[], &mut interpreter).unwrap();
        let shape = Shape::new_axis_plane(2.5, veclib::Vec3Axis::Y, math::csg::CSGType::Union)
            .new(&[p], &mut interpreter)
            .unwrap();
        let d = Noise::default()
            .set_type(NoiseType::Simplex)
            .set_inverted(false)
            .set_strength(120.0)
            .set_scale(0.003)
            .new(&[p], &mut interpreter)
            .unwrap();
        let c = DensityOperation::Addition.new(&[shape, d], &mut interpreter).unwrap();
        interpreter
    }
    // Add a specific node to the system
    pub fn add<T: NodeInterpreter + 'static>(&mut self, node_interpreter: T, mut getter: VarHashGetter) -> Result<VarHash, InterpreterError> {
        let id = self.vars.len();
        // Create the var hash
        let boxed = Box::new(node_interpreter);
        let var_hash = VarHash {
            index: id,
            _type: boxed.get_output_type(&getter),
            passed_data: PassedData::default(),
        };
        // Create a variable for this node
        let line = format!(
            "{} {} = {};",
            var_hash._type.to_glsl_type(),
            boxed.custom_name(var_hash.get_name()),
            boxed.get_node_string(&getter)?
        );
        // Add the line for this specific node (might get removed later on if this node was not used)
        self.lines.push(line);
        // Add the var hash for this specific node
        self.vars.push(var_hash);
        // Make the inputs of this node considered as used nodes
        for x in getter.inputs_indices.iter() {
            if !self.used_nodes.contains(x) {
                self.used_nodes.push(*x);
            }
        }
        // Le self var hash
        getter.self_varhash = Some(var_hash);
        // Create a node
        let node = Node { getter, node_interpreter: boxed };
        // Add the node
        self.nodes.push(node);
        Ok(*self.vars.get(id).unwrap())
    }
    // Finalize the tree with a specific var hash
    pub fn finalize(&mut self) -> Option<(String, CSGTree)> {
        // We cannot finalize this node if it was already finalized
        if self.finalized {
            return None;
        }
        // We are going to use the last node as the final node
        let final_density_varhash = self.vars.last().unwrap().clone();
        // Check if the supplied varhash is a of type "density"
        match &final_density_varhash._type {
            VarHashType::Density => {
                // We can continue
                self.finalized = true;
                FinalDensity.new(&[final_density_varhash], self).unwrap();
            }
            _ => {
                /* No good */
                panic!();
            }
        }
        // Getting the GLSL code here
        let string = self.lines.join("\n");

        // Getting the CSGTree now
        // Default CSGTree is based around our first "base_density" node, since it is a shape node undercover
        let mut csgtree: CSGTree = CSGTree::default();
        let mut input_ranges: HashMap<usize, (f32, f32)> = HashMap::new();
        input_ranges.insert(0, (0.0, 0.0));
        // Start from the oldest nodes
        for i in 1..self.used_nodes.len() {
            let x = self.used_nodes[i];
            let node = self.nodes.get(x).unwrap();
            // Get the variables from the node
            let mut getter = node.getter.clone();
            // Make sure the getter has updated values
            for (local_index, global_index) in getter.inputs_indices.iter().enumerate() {
                getter.inputs[local_index] = *self.vars.get(*global_index).unwrap();
            }
            let output_var = self.vars.get_mut(x).unwrap();
            let new_input_ranges = getter.inputs_indices.iter().map(|x| input_ranges.get(x).unwrap().clone()).collect::<Vec<(f32, f32)>>();
            // Gotta calculate the range first
            let range = node.node_interpreter.calculate_range(&getter, new_input_ranges);
            input_ranges.insert(output_var.index, range);
            // Update the csg tree
            node.node_interpreter.update_csgtree(&mut output_var.passed_data, &getter, &mut csgtree, range);
        }
        Some((string, csgtree))
    }
}
// Custom gets
impl Interpreter {
    // Get base density node
    // Get base shape node
    pub fn get_base_shape_node(&self) -> (VarHash, &Node, &Box<dyn NodeInterpreter>) {
        // Check if we are even valid in the first place
        if !self.finalized {
            panic!()
        }
        // The index for this node is "0"
        let var_hash = self.vars.get(1).unwrap();
        let node = self.nodes.get(1).unwrap();
        (*var_hash, &node, &node.node_interpreter)
    }
}
