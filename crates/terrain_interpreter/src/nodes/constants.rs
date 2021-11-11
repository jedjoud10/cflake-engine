use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};
pub enum Constants {
    Float(f32),
    Vec2(veclib::Vector2<f32>),
    Vec3(veclib::Vector3<f32>),
}

impl NodeInterpreter for Constants {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Get the GLSL name of the operation and combine with the two inputs
        Ok(match self {
            Constants::Float(x) => format!("{}", x),
            Constants::Vec2(x) => format!("vec2({}, {})", x.x, x.y),
            Constants::Vec3(x) => format!("vec3({}, {}, {})", x.x, x.y, x.z),
        })
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> crate::var_hash::VarHashType {
        match self {
            Constants::Float(_) => VarHashType::Density,
            Constants::Vec2(_) => VarHashType::Vec2,
            Constants::Vec3(_) => VarHashType::Vec3,
        }
    }
}
