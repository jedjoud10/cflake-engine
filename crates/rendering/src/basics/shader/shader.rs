use super::{compile_shader, ShaderInitSettings, SharedExpansionData, PreCompilationData};

use crate::object::ObjectSealed;

use ahash::AHashSet;
use getset::Getters;

use super::{load_includes, IncludeExpansionError, Program};

// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
#[derive(Getters)]
pub struct Shader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    program: Option<Program>,

    // Pre-compilation data
    pre: Option<PreCompilationData>,
}

impl Shader {
    // Creates a new shader using some shader init settings
    // TODO: Implement a global shader cache 
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Loop through the shader sources and modify/expand them
        let mut sources = std::mem::take(settings.sources_mut());
        for (_, source) in sources.iter_mut() {
            // Data that keeps track of was was expanded and what wasn't
            let mut shared = SharedExpansionData::default();
            // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
            // Include the includables until they cannot be included
            while load_includes(&settings, source.text_mut(), &mut shared)? {
                // We are still including paths
            }
        }

        // Create the pre-compilation data 
        let pre = PreCompilationData {
            sources,
        };

        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: None,
            pre: Some(pre),
        })
    }
}

impl ObjectSealed for Shader {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        self.program = Some(compile_shader(self.pre.take().unwrap()));
    }

    fn disposed(self) {
        unsafe {
            gl::DeleteProgram(self.program.unwrap().name());
        }
    }
}
