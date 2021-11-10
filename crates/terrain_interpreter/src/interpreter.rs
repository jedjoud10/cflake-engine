use math::{bounds::AABB, constructive_solid_geometry::CSGTree};

use crate::{Influence, Node, NodeInterpreter, error::InterpreterError, nodes::*, var_hash::{VarHash, VarHashType}, var_hash_getter::VarHashGetter};

// The main system that will be made from multiple densities and combiners
pub struct Interpreter {
    pub nodes: Vec<Node>,
    pub used_nodes: Vec<usize>,
    pub influences: Vec<Influence>,
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
            influences: Vec::new(),
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
        let length = VectorOperations::Length.new(&[p], &mut interpreter).unwrap();
        let constant = Constants::Float(200.0).new(&[], &mut interpreter).unwrap();
        let length2 = DensityOperation::Subtraction.new(&[length, constant], &mut interpreter).unwrap();
        let y = Splitter::Y.new(&[p], &mut interpreter).unwrap();
        let snoise = Noise::default().new(&[p], &mut interpreter).unwrap();
        let final_node = DensityOperation::Addition.new(&[length2, snoise], &mut interpreter).unwrap();
        interpreter.finalize(final_node);
        interpreter
    }
    // Add a specific node to the system
    pub fn add<T: NodeInterpreter + 'static>(&mut self, node_interpreter: T, getter: VarHashGetter) -> Result<VarHash, InterpreterError> {
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

        // Create a node
        let node = Node {
            getter,
            node_interpreter: boxed,
        };
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
        // We will start from the finalized density node and go down the tree, and keep track of the nodes with non-null influence
        let mut base_csgtree: CSGTree = CSGTree::default();
        let mut base_influence: Influence = Influence::new_base();
        for x in self.used_nodes.iter() {
            let node = self.nodes.get(*x).unwrap();
            // Get the variables from the node
            let getter = &node.getter;
            let output_var = self.vars.get(*x).unwrap();
            // Calculate the influence now 
            /*
            match node.node_interpreter.calculate_influence(&node.getter, ) {
                Some(x) => {
                    // Some cool shit happens here idk what yet though
                },
                None => { /* No influence calculated for this node */ },
            }
            */
        }
        None
    }
}
