use super::{compile_shader, ShaderInitSettings};

use crate::object::PipelineElement;

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

impl PipelineElement for Shader {
    fn add(self, pipeline: &mut crate::pipeline::Pipeline) -> crate::pipeline::Handle<Self> {
        // Compiling
        self.program = compile_shader(self.settings.sources_mut());
        pipeline.shaders.insert(self)
    }

    fn find<'a>(pipeline: &'a crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a Self> {
        pipeline.shaders.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a mut Self> {
        pipeline.shaders.get_mut(handle)
    }

    fn disposed(self) {
        unsafe {
            gl::DeleteProgram(self.program.program());
        }
    }
}
