use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// A some noise node
#[derive(Debug)]
pub struct Noise {
    pub strength: f32,
    pub scale: f32,
    pub inverted: bool,
    pub _type: NoiseType,
}
impl Noise {
    // New
    pub fn new() -> Self {
        Self::default()
    }
    // Set type
    pub fn set_type(mut self, _type: NoiseType) -> Self {
        self._type = _type;
        self
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
    // Set inverted
    pub fn set_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
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
            inverted: false,
        }
    }
}

impl NodeInterpreter for Noise {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check input real quick
        let input = getter.get(0, VarHashType::Vec3)?;
        // Create the GLSL string for this node, so we can make a variable out of it
        let main = match self._type {
            NoiseType::Simplex => format!("snoise({} * {})", input.get_name(), self.scale),
            NoiseType::VoronoiSimplex => todo!(),
            NoiseType::VoronoiDistance => format!("voronoi({} * {}).x", input.get_name(), self.scale),
            NoiseType::VoronoiDistance2 => todo!(),
            NoiseType::VoronoiCell => format!("voronoi({} * {}).z", input.get_name(), self.scale),
        }.to_string();
        // Check if we have to invert if
        let string: String;
        if self.inverted {
            string = format!("-{} * {}", main, self.strength);
        } else {
            // Not inverted, we just have to multiply by the strength
            string = format!("{} * {}", main, self.strength);
        }
        Ok(string)
    }
    fn get_output_type(&self, _inputs: &VarHashGetter) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
    fn calculate_range(&self, getter: &VarHashGetter, input_ranges: Vec<(f32, f32)>) -> (f32, f32) {
        (-self.strength, self.strength)
    }
}
