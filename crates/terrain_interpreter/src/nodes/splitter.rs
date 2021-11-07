use crate::{
    error::InterpreterError,
    var_hash::{VarHash, VarHashType},
    Influence, NodeInterpreter,
};

// How we split the vectors
#[derive(Debug)]
pub enum Splitter {
    // Split values
    X,
    Y,
    // This is only valid for the Vec3s
    Z,
}

impl NodeInterpreter for Splitter {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Check if we can even split this varhash input
        let input = inputs.get(0).ok_or(InterpreterError::missing_input(0, self))?;
        match input._type {
            VarHashType::Vec2 => match self {
                Splitter::Z => return Err(InterpreterError::input_err(input, 0, self, VarHashType::Vec3)),
                _ => {}
            },
            VarHashType::Vec3 => {}
            _ => return Err(InterpreterError::input_err(input, 0, self, VarHashType::Vec2)),
        };
        // Split the input
        Ok(match self {
            Splitter::X => format!("{}.x", input.get_name()),
            Splitter::Y => format!("{}.y", input.get_name()),
            Splitter::Z => format!("{}.z", input.get_name()),
        })
    }
    fn get_output_type(&self) -> VarHashType {
        VarHashType::Density
    }
}
