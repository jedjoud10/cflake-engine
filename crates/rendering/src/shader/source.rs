use crate::context::{Context, ToGlType};
use assets::{loader::AssetLoader, Asset};
use std::num::NonZeroU32;

// The type of each shader source
#[derive(Clone, Copy)]
pub enum SourceStage {
    Vertex,
    Fragment, 
    Compute
    /*
    TessControl,
    TessEval,
    Geometry
    */
}

impl ToGlType for SourceStage {
    fn target(&self) -> NonZeroU32 {
        unsafe { match self {
                SourceStage::Vertex => NonZeroU32::new_unchecked(gl::VERTEX_SHADER),
                SourceStage::Fragment => NonZeroU32::new_unchecked(gl::FRAGMENT_SHADER),
                SourceStage::Compute => NonZeroU32::new_unchecked(gl::COMPUTE_SHADER),
            }
        }
    }
}

// A single shader source that can make up a bigger shader. A source is usually a single text file ending with .glsl
pub struct Source {
    // The cleaned and processed shader source text (without any directives)
    txt: String,

    // The source's shader stage
    stage: SourceStage,
}

// This is the raw GLSL text that we load from the shader files
struct RawSource(String);

impl Asset<'static> for RawSource {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["vert.glsl", "frag.glsl", "cmpt.glsl", "func.glsl"]
    }

    fn deserialize<'loader>(bytes: assets::loader::AssetBytes, args: Self::Args) -> Self {
        RawSource(String::from_utf8(bytes.as_ref().to_vec()).unwrap())
    }
}

impl<'a> Asset<'a> for Source {
    type Args = &'a mut Context;

    fn extensions() -> &'static [&'static str] {
        &["vert.glsl", "frag.glsl", "cmpt.glsl"]
    }

    fn deserialize<'loader>(bytes: assets::loader::AssetBytes, args: Self::Args) -> Self {
        todo!()
    }
}
