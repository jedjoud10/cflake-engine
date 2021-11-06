use crate::{Influence, NodeInterpreter, var_hash::VarHash};

// A Simplex-Noise node
pub struct SNoise {
    pub strength: f32,
    pub scale: f32,
}

impl Default for SNoise {
    fn default() -> Self {
        Self {
            strength: 1.0, 
            scale: 0.001
        }
    }
}

impl NodeInterpreter for SNoise {
    fn get_node_string(&self, inputs: Vec<VarHash>) -> String {
        // Create the HLSL string for this node, so we can make a variable out of it
        format!("snoise({} * {}) * {}", inputs[0].get_name(), self.scale, self.strength)
    }

    fn calculate_influence(&self) -> Influence {
        // Modified influence
        // Normal range is between -1, 1
        Influence::Modified(-self.strength, self.strength)
    }

    fn get_output_type(&self) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}