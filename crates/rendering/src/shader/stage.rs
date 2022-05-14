use assets::Asset;

use super::RawText;

// This trait is implemented for each shader stage, like the vertex stage or fragment stage
// TODO: Rename
pub trait StageDescriptor where Self: Sized {
    const GL_TYPE: u32;
    const EXTENSION: &'static str;

    // Convert a string into a valid stage descriptor
    fn from_string(string: String) -> Self;
} 

// Simple stage wrapper
pub struct Stage<T: StageDescriptor>(T);

impl<T: StageDescriptor> From<String> for Stage<T> {
    fn from(string: String) -> Self {
        Self(T::from_string(string))
    }
}

impl<T: StageDescriptor> From<&str> for Stage<T> {
    fn from(str: &str) -> Self {
        Self(T::from_string(str.to_string()))
    }
}

// Automatically implemented the asset trait for stages
impl<T: StageDescriptor> Asset<'static> for Stage<T> {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &[T::EXTENSION]
    }

    fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
        Self::from(RawText::deserialize(bytes, args).0)
    }
}

// A vertex stage that will be loaded from .vrsh files
pub struct Vertex(String);
impl StageDescriptor for Vertex {
    const GL_TYPE: u32 = gl::VERTEX_SHADER;
    const EXTENSION: &'static str = ".vrsh.glsl";

    fn from_string(string: String) -> Self {
        Self(string)
    }
}

// A fragment stage that will be loaded from .frsh files
pub struct Fragment(String);
impl StageDescriptor for Fragment {
    const GL_TYPE: u32 = gl::FRAGMENT_SHADER;
    const EXTENSION: &'static str = ".frsh.glsl";

    fn from_string(string: String) -> Self {
        Self(string)
    }
}


// A compute stage (only for compute shaders) that will be loaded from .cmpt files
pub struct Compute(String);
impl StageDescriptor for Compute {
    const GL_TYPE: u32 = gl::COMPUTE_SHADER;
    const EXTENSION: &'static str = ".cmpt.glsl";

    fn from_string(string: String) -> Self {
        Self(string)
    }
}

// Type aliases cause I'm very cool
pub type VertexStage = Stage<Vertex>;
pub type FragmentStage = Stage<Fragment>;
pub type ComputeStage = Stage<Compute>; 