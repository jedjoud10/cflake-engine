use math::constructive_solid_geometry::CSGTree;

use crate::{
    error::InterpreterError,
    nodes::*,
    var_hash::{VarHash, VarHashType},
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
    // New
    pub fn new() -> Self {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();
        // Add the default pos.y nodes
        let p = BasePosition::default().new(&[], &mut interpreter).unwrap();
        let shape = Shape::new_cube(veclib::Vector3::Y*10.0, veclib::Vector3::ONE*10.0, math::csg::CSGType::Union)
            .new(&[p], &mut interpreter)
            .unwrap();
        let d = BaseDensity::default().new(&[shape], &mut interpreter).unwrap();
        let s = Noise::default().new(&[p], &mut interpreter).unwrap();
        let c = DensityOperation::Addition.new(&[d, s], &mut interpreter).unwrap();
        interpreter.finalize(c);
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
            self.used_nodes.push(*x);
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
    pub fn finalize(&mut self, final_density_varhash: VarHash) {
        // Check if the supplied varhash is a of type "density"
        match &final_density_varhash._type {
            VarHashType::Density => {
                // We can continue
                self.finalized = true;
                FinalDensity::default().new(&[final_density_varhash], self).unwrap();
            }
            _ => {
                /* No good */
                panic!("Finalized node is not of type '{:?}'!", VarHashType::Density)
            }
        }
    }
    // Read back the GLSL data from this interpreter
    pub fn read_glsl(&self) -> Option<String> {
        // Check if we finalized
        if !self.finalized {
            return None;
        }
        //lines.reverse();
        Some(self.lines.join("\n"))
    }
    // Read back the CSG tree
    pub fn read_csgtree(&self) -> Option<CSGTree> {
        // Default CSGTree is based around our first "base_density" node, since it is a shape node undercover
        let mut csgtree: CSGTree = CSGTree::default();
        let (_, node, bsn) = self.get_base_shape_node();
        // Calculate the base influence
        bsn.update_csgtree(&node.getter, &mut csgtree, (0.0, 0.0));
        let mut input_ranges: Vec<(f32, f32)> = Vec::new();
        input_ranges.push((0.0, 0.0));
        // Start from the oldest nodes
        for i in 1..self.used_nodes.len() {
            let x = self.used_nodes[i];
            let node = self.nodes.get(x).unwrap();
            // Get the variables from the node 
            let getter = &node.getter;
            let output_var = self.vars.get(x).unwrap();
            for x in getter.inputs_indices.iter() {
                println!("Test: {}", x);
            }
            let new_input_ranges = getter.inputs_indices.iter().map(|x| input_ranges.get(*x).unwrap().clone()).collect::<Vec<(f32, f32)>>();
            // Gotta calculate the range first
            for x in new_input_ranges.iter() {
                println!("Input: [{}, {}]", x.0, x.1);
            }
            let range = node.node_interpreter.calculate_range(getter, new_input_ranges);
            println!("Add input range: [{}, {}] at index {}, instead of index {}", range.0, range.1, input_ranges.len(), output_var.index);
            input_ranges.insert(output_var.index, range);
            // Update the csg tree
            node.node_interpreter.update_csgtree(getter, &mut csgtree, range);
        }
        Some(csgtree)
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
        let var_hash = self.vars.get(0).unwrap();
        let node = self.nodes.get(0).unwrap();
        (*var_hash, &node, &node.node_interpreter)
    }
}
