// This trait will be implemented on objects that contain a raw OpenGL name
pub trait ToGlName {
    fn name(&self) -> u32;
}

// This trait will be implemented on objects that have a unique OpenGL type
pub trait ToGlTarget {
    fn target() -> u32;
}

// Objects that can be shared/sent to the GPU through OpenGL functions
pub trait Shared: Copy + Sized + Sync + Send + 'static {}
impl<T: Copy + Sized + Sync + Send + 'static> Shared for T {}
