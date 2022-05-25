use super::{FragmentStage, Program, VertexStage};

// A shader that will render our objects onto the screen
// This will make use of two shader stages, the vertex stage, and fragment stage
pub struct Shader(pub(super) Program);

impl AsRef<Program> for Shader {
    fn as_ref(&self) -> &Program {
        &self.0
    }
}

impl AsMut<Program> for Shader {
    fn as_mut(&mut self) -> &mut Program {
        &mut self.0
    }
}
