use crate::{Influence, NodeInterpreter, var_hash::{VarHash, VarHashType}};

// How we split the vectors
pub enum Splitter {
    // Split values
    X, Y,
    // This is only valid for the Vec3s
    Z
}

impl NodeInterpreter for Splitter {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> String {
        // Split the input
        match self {
            Splitter::X => format!("{}.x", inputs[0].get_name()),
            Splitter::Y => format!("{}.y", inputs[1].get_name()),
            Splitter::Z => format!("{}.z", inputs[2].get_name()),
        }
    }
    fn calculate_influence(&self, inputs: &Vec<Influence>) -> Influence {
        // TODO: This
        todo!()
    }
    fn get_output_type(&self) -> VarHashType {
        VarHashType::Density
    }
}