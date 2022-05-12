use std::num::NonZeroU32;
use assets::{Asset, loader::AssetLoader};
use crate::context::Context;

// A single shader source that can make up a bigger shader. A source is usually a single text file ending with .glsl
pub struct Source {
    // The filtered shader source text (without any directives)
    txt: String,

    // OpenGL type for this shader source
    gltype: NonZeroU32,
}

// A source shard is simply some piece of code that might get included externally by custom code or internally using include directives
struct Shard(String);

impl Asset<'static> for Shard {
    type Args = ();

    fn is_extension_valid(extension: &str) -> bool {
        match extension {
            "vert.glsl" | "frag.glsl" | "cmpt.glsl" | "func.glsl" => true,
            _ => false
        }
    }

    fn deserialize<'loader>(bytes: assets::loader::AssetBytes, args: Self::Args) -> Self {
        Shard(String::from_utf8(bytes.as_ref().to_vec()).unwrap())
    }
}

// Process some shader source code, expanding certain directives and reducing constants to their literal values
fn process_shader_shard(base: Shard, loader: &mut AssetLoader) -> Result<Shard, assets::LoadError> {
    todo!()
}


impl<'a> Asset<'a> for Source {
    type Args = &'a mut Context;

    fn is_extension_valid(extension: &str) -> bool {
        match extension {
            "vert.glsl" | "frag.glsl" | "cmpt.glsl" => true,
            _ => false
        }
    }

    fn try_load_with(loader: &mut AssetLoader, path: &str, args: Self::Args) -> Result<Self, assets::LoadError> {
        let base = Shard::try_load(loader, path)?;
        
    }

    fn deserialize<'loader>(bytes: assets::loader::AssetBytes, args: Self::Args) -> Self {
        todo!()
    }

}