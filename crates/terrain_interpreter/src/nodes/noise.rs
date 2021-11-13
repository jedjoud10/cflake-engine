use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// A some noise node
#[derive(Debug)]
pub struct Noise {
    pub strength: f32,
    pub scale: f32,
    pub _type: NoiseType,
}
impl Noise {
    // New
    pub fn new() -> Self {
        Self::default()
    }
    // Set strength
    pub fn set_strength(mut self, strength: f32) -> Self {
        self.strength = strength;
        return self;
    }
    // Set scale
    pub fn set_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        return self;
    }
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
            strength: 80.0,
            scale: 0.005,
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
    fn calculate_range(&self, getter: &VarHashGetter, input_ranges: Vec<(f32, f32)>) -> (f32, f32) {
        (-self.strength, self.strength)
    }
}
