// This is a wrapper that allows the user to send data to GPU shaders
// in a clean and safe fashion
pub struct Uniforms {
}

impl Uniforms {
    // Set a push constants block without checking safety
    pub unsafe fn set_push_constants_unchecked<T: Bindings>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> Option<()> {
        todo!()
    }

    // Set a push constants block and make sure it's valid
    pub fn set_push_constants<T>(
        &mut self,
        name: &'static str,
        value: &T
    ) -> Option<()> {
        todo!()
    }
}

// This trait will store the layout definition of a specific struct and
// allow the user to make sure the given layout in Uniforms matches up
// with the layout within the graphical pipeline (safety)
pub trait Bindings {
}