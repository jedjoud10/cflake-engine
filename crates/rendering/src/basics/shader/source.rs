use assets::Asset;

// A shader source that has been loaded from a shader file (.glsl)
#[derive(Default, Clone)]
pub struct ShaderSource {
    // File info
    file: String,
    text: String,
    // And a specific type just to help use
    _type: ShaderSourceType,
}

// Getters and mut getters
impl ShaderSource {
    pub fn file(&self) -> &str {
        self.file.as_str()
    }
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
    pub(crate) fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }
    pub fn _type(&self) -> ShaderSourceType {
        self._type
    }
}

impl Default for ShaderSourceType {
    fn default() -> Self {
        Self::Vertex
    }
}

// Shader source type
#[derive(Clone, Copy, Debug)]
pub enum ShaderSourceType {
    Vertex,
    Fragment,
    Compute,
}

impl Asset for ShaderSource {
    fn deserialize(self, meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Load a shader source
        // Load a shader source from scratch
        let text = String::from_utf8(bytes.to_vec()).ok()?;
        let extension = meta.name.to_str().unwrap().to_string().split('.').map(|x| x.to_string()).collect::<Vec<_>>()[1..].join(".");
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
