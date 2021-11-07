use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// A some noise node
#[derive(Debug)]
pub struct Noise {
    pub strength: f32,
    pub scale: f32,
    pub _type: NoiseType,
}
// Some noise type
#[derive(Debug)]
pub enum NoiseType {
    Simplex,
    VoronoiSimplex,
    VoronoiDistance,
    VoronoiDistance2,
    VoronoiCell,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            strength: 40.0,
            scale: 0.001,
            _type: NoiseType::Simplex,
        }
    }
}

impl NodeInterpreter for Noise {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check input real quick
        let input = getter.get(0, VarHashType::Vec3)?;
        // Create the GLSL string for this node, so we can make a variable out of it
        Ok(format!("snoise({} * {}) * {}", input.get_name(), self.scale, self.strength))
    }
    fn get_output_type(&self, _inputs: &VarHashGetter) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}
