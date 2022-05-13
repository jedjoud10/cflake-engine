use crate::context::{Context, ToGlType};
use assets::{loader::{AssetLoader}, Asset};
use std::{num::NonZeroU32, borrow::Cow, path::PathBuf};

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

// This is a raw source that simply contains the raw GLSL code given from the file. This code is unprocessed, so it might contain preprocessor directives
pub struct RawSource {
    // The raw text given by the source
    raw_txt: String,

    // The path from where we loaded this raw sources
    path: PathBuf,
}

impl Asset<'static> for RawSource {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["vert.glsl", "frag.glsl", "cmpt.glsl", "func.glsl"]
    }

    fn deserialize(bytes: &[u8], path: std::path::PathBuf, args: Self::Args, ctx: assets::loader::LoadingContext) -> Self {
        Self { raw_txt: String::from_utf8(bytes.as_ref().to_vec()).unwrap(), path }
    }
}