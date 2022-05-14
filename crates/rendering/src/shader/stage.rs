use assets::Asset;

use super::RawText;

// This trait is implemented for each shader stage, like the vertex stage or fragment stage
pub trait Stage: Sized + From<String> + AsRef<str> {
    const GL_TYPE: u32;
    const EXTENSION: &'static str;
} 

// A vertex stage that will be loaded from .vrsh files
pub struct VertexStage(String);

impl From<String> for VertexStage {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for VertexStage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Stage for VertexStage {
    const GL_TYPE: u32 = gl::VERTEX_SHADER;
    const EXTENSION: &'static str = ".vrsh.glsl";
}

impl Asset<'static> for VertexStage {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &[Self::EXTENSION]
    }

    fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
        Self::from(RawText::deserialize(bytes, args).0)
    }
}

// A fragment stage that will be loaded from .frsh files
pub struct FragmentStage(String);

impl From<String> for FragmentStage {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for FragmentStage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Stage for FragmentStage {
    const GL_TYPE: u32 = gl::FRAGMENT_SHADER;
    const EXTENSION: &'static str = ".frsh.glsl";
}

impl Asset<'static> for FragmentStage {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &[Self::EXTENSION]
    }

    fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
        Self::from(RawText::deserialize(bytes, args).0)
    }
}

// A compute stage (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeStage(String);

impl From<String> for ComputeStage {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for ComputeStage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Stage for ComputeStage {
    const GL_TYPE: u32 = gl::COMPUTE_SHADER;
    const EXTENSION: &'static str = ".cmpt.glsl";
}

impl Asset<'static> for ComputeStage {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &[Self::EXTENSION]
    }

    fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
        Self::from(RawText::deserialize(bytes, args).0)
    }
}