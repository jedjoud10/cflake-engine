use math::bounds::AABB;

use crate::{Influence, NodeInterpreter, error::InterpreterError, nodes::*, var_hash::{VarHash, VarHashType}, var_hash_getter::VarHashGetter};

// The main system that will be made from multiple densities and combiners
pub struct Interpreter {
    pub nodes: Vec<Box<dyn NodeInterpreter>>,
    pub vars: Vec<VarHash>,
    pub lines: Vec<String>,
    pub finalized: bool,
    pub max_influence: Influence,
}

impl Default for Interpreter {
    fn default() -> Self {
        // Create the default starter node
        Self {
            nodes: Vec::new(),
            vars: Vec::new(),
            lines: Vec::new(),
            finalized: false,
            max_influence: Influence::Modified(-1.0, 1.0),
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
        let y = Splitter::Y.new(&[p], &mut interpreter).unwrap();
        interpreter.finalize(y);
        interpreter
    }
    // Add a specific node to the system
    pub fn add<T: NodeInterpreter>(&mut self, node_interpreter: T, getter: VarHashGetter) -> Result<VarHash, InterpreterError> {
        let id = self.vars.len();
        // Create the var hash
        let boxed = Box::new(node_interpreter);
        let var_hash = VarHash {
            name: id,
            _type: boxed.get_output_type(&getter),
        };
        self.vars.push(var_hash);
        // Create a variable for this node
        let line = format!(
            "{} {} = {};",
            var_hash._type.to_glsl_type(),
            boxed.custom_name(var_hash.get_name()),
            boxed.get_node_string(&getter)?
        );
        self.lines.push(line);
        Ok(*self.vars.get(id).unwrap())
    }
    // Finalize the tree with a specific var hash
    pub fn finalize(&mut self, final_density_varhash: VarHash) {
        // Check if the supplied varhash is a of type "density"
        match &final_density_varhash._type {
            VarHashType::Density => {
                // We can continue
                self.finalized = true;
                FinalDensity::default().new(&[final_density_varhash], self);
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
    // Read back the bound intersection AABB for this interpreter
    pub fn read_aabb(&self) -> Option<AABB> {
        None
    }
}
