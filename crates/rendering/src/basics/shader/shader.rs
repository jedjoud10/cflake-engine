use super::{compile_shader, ShaderInitSettings};

use crate::object::Object;

use ahash::AHashSet;
use getset::Getters;

use super::{load_includes, IncludeExpansionError, ShaderProgram};

// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
#[derive(Getters)]
pub struct Shader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    program: ShaderProgram,
    // Init settings
    #[getset(get = "pub")]
    settings: ShaderInitSettings,
}

impl Shader {
    // Creates a new shader using some shader init settings
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(settings.sources_mut());
        for (_, source) in sources.iter_mut() {
            let mut included_paths: AHashSet<String> = AHashSet::new();
            // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
            // Include the includables until they cannot be included
            while load_includes(&settings, source.text_mut(), &mut included_paths)? {
                // We are still including paths
            }
        }
        *settings.sources_mut() = sources;

        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: Default::default(),
            settings,
        })
    }
}

impl Object for Shader {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        self.program = compile_shader(self.settings.sources_mut());
    }

    fn disposed(self) {
        unsafe {
            gl::DeleteProgram(self.program.program());
        }
    }
}
