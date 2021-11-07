use crate::{error::InterpreterError, var_hash::VarHash, Influence, NodeInterpreter};

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
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Check input real quick
        let input = inputs.get(0).ok_or(InterpreterError::missing_input(0, self))?;
        match input._type {
            crate::var_hash::VarHashType::Vec3 => {}
            _ => return Err(InterpreterError::input_err(input, 0, self, crate::var_hash::VarHashType::Vec3)),
        }
        // Create the GLSL string for this node, so we can make a variable out of it
        Ok(format!("snoise({} * {}) * {}", input.get_name(), self.scale, self.strength))
    }
    fn get_output_type(&self) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}
