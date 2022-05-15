use crate::context::{Cached, ToGlName};

use super::{FragmentStage, VertexStage, Program};

// A shader that will render our objects onto the screen
// This will make use of two shader stages, the vertex stage, and fragment stage
pub struct Shader(Program);

impl Cached for Shader {}
impl AsRef<Program> for Shader {
    fn as_ref(&self) -> &Program {
        &self.0
    }
}