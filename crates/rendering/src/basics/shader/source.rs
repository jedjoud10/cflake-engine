use assets::Asset;
use getset::{Getters, MutGetters};

// A shader source that has been loaded from a shader file (.glsl)
#[derive(Default, Clone, Getters, MutGetters)]
pub struct ShaderSource {
    // File info
    #[getset(get = "pub")]
    file: String,
    #[getset(get = "pub", get_mut = "pub(crate)")]
    text: String,
    // And a specific type just to help use
    #[getset(get = "pub")]
    _type: ShaderSourceType,
}

impl Default for ShaderSourceType {
    fn default() -> Self {
        Self::Vertex
    }
}

// Shader source type
#[derive(Clone, Debug)]
pub enum ShaderSourceType {
    Vertex,
    Fragment,
    Compute,
}

impl Asset for ShaderSource {
    fn deserialize(self, meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized {
        // Load a shader source
        // Load a shader source from scratch
        let text = String::from_utf8(bytes.to_vec()).ok()?;
        let extension = meta.name.to_str().unwrap().to_string().split(".").map(|x| x.to_string()).collect::<Vec<_>>()[1..].join(".");
        Some(ShaderSource {
            file: meta.name.to_str().unwrap().to_string(),
            text,
            _type: match extension.as_str() {
                "vrsh.glsl" => ShaderSourceType::Vertex,
                "frsh.glsl" => ShaderSourceType::Fragment,
                "cmpt.glsl" => ShaderSourceType::Compute,
                _ => panic!(),
            },
        })
    }
}