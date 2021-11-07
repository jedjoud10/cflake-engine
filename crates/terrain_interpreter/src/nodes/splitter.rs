use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}, var_hash_getter::VarHashGetter};

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
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check if we can even split this varhash input
        let t = getter.get(0, VarHashType::Vec2).or(getter.get(0, VarHashType::Vec3))?;
        match t._type {
            VarHashType::Vec2 => { /* We must do additional checks, if they fail that means that we tried to read the Z value of a Vec2 */
                match self {
                    Splitter::Z => return Err(InterpreterError::new("Tried to read the Z value of a Vec2!")),
                    _ => { /* Dis fine */ }    
                }
            },
            VarHashType::Vec3 => { /* We are fine here because even if we want the Z axis of this value we can safely return it */ },
            _ => { /* This should never happen */ }
        }
        let name = t.get_name();
        // Split the input
        Ok(match self {
            Splitter::X => format!("{}.x", name),
            Splitter::Y => format!("{}.y", name),
            Splitter::Z => format!("{}.z", name),
        })
    }
    fn get_output_type(&self, getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
}
