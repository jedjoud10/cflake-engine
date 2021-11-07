use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}, var_hash_getter::VarHashGetter};

// A Simplex-Noise node
#[derive(Debug)]
pub struct SNoise {
    pub strength: f32,
    pub scale: f32,
}

impl Default for SNoise {
    fn default() -> Self {
        Self { strength: 1.0, scale: 0.001 }
    }
}

impl NodeInterpreter for SNoise {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check input real quick
        let input = getter.get(0, VarHashType::Vec3)?;
        // Create the GLSL string for this node, so we can make a variable out of it
        Ok(format!("snoise({} * {}) * {}", input.get_name(), self.scale, self.strength))
    }
    fn get_output_type(&self, inputs: &VarHashGetter) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}
