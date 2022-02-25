use gl::types::GLuint;

use crate::basics::shader::query_shader_uniforms_definition_map;
use crate::object::{
    Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject,
};
use crate::pipeline::Pipeline;

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::null;

use super::{load_includes, IncludeExpansionError};

// Shader source type
pub(crate) enum ShaderSourceType {
    Vertex,
    Fragment,
    Compute,
}
// And a shader source
pub(crate) struct ShaderSource {
    // Corresponding path for this shader source, since we store them in different files
    pub path: String,
    // The actual source code text
    pub text: String,
    // And a specific type just to help use
    pub _type: ShaderSourceType,
}

// Some shader settings that we can use to load the shader
pub struct ShaderSettings {
    // Some external code that we can load directly onto the shader
    pub(crate) external_code: HashMap<String, String>,
    pub(crate) consts: HashMap<String, String>,
    pub(crate) sources: HashMap<String, ShaderSource>,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        // Some actual default consts names and their values
        let consts = HashMap::new();

        Self {
            external_code: Default::default(),
            consts,
            sources: Default::default(),
        }
    }
}

impl ShaderSettings {
    // Load some external code that can be loading using specific include points
    pub fn external_code(mut self, id: &str, string: String) -> Self {
        self.external_code.insert(id.to_string(), string);
        self
    }
    // Load some shader constants that can be loaded directly while compiling the shader
    pub fn shader_constant<T: ToString>(mut self, id: &str, val: T) -> Self {
        self.consts.insert(id.to_string(), val.to_string());
        self
    }
    // Load a shader source
    pub fn source(mut self, path: &str) -> Self {
        // Load a shader source from scratch
        let metadata = assets::metadata::AssetMetadata::new(path).unwrap();
        let text = assets::assetc::load::<String>(path).unwrap();
        let extension = metadata
            .name
            .to_str()
            .unwrap()
            .to_string()
            .split(".")
            .map(|x| x.to_string())
            .collect::<Vec<_>>()[1..]
            .join(".");
        self.sources.insert(
            path.to_string(),
            ShaderSource {
                path: path.to_string(),
                text,
                _type: match extension.as_str() {
                    "vrsh.glsl" => ShaderSourceType::Vertex,
                    "frsh.glsl" => ShaderSourceType::Fragment,
                    "cmpt.glsl" => ShaderSourceType::Compute,
                    _ => panic!(),
                },
            },
        );
        self
    }
}

// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
pub struct Shader {
    // The OpenGL program linked to this shader
    pub program: GLuint,
    // The updated and modified shader sources
    pub(crate) sources: HashMap<String, ShaderSource>,
}
impl PipelineObject for Shader {
    // Reserve an ID for this shader
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.shaders.gen_id()))
    }
    // Send this shader to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Shader(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Shader(Deconstruct::<Self>(id))
    }
    // Add the shader to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Compile the shader first
        // Compile a single shader source
        fn compile_single_source(source_data: ShaderSource) -> u32 {
            let shader_type: u32;
            println!("Compiling & Creating Shader Source {}...", source_data.path);
            match source_data._type {
                ShaderSourceType::Vertex => shader_type = gl::VERTEX_SHADER,
                ShaderSourceType::Fragment => shader_type = gl::FRAGMENT_SHADER,
                ShaderSourceType::Compute => {
                    panic!()
                } // We are not allowed to create compute shaders using the normal create_shader function
            }
            unsafe {
                let program = gl::CreateShader(shader_type);
                // Compile the shader
                let cstring = CString::new(source_data.text.clone()).unwrap();
                let shader_source: *const i8 = cstring.as_ptr();
                gl::ShaderSource(program, 1, &shader_source, null());
                gl::CompileShader(program);
                // Check for any errors
                let mut info_log_length: i32 = 0;
                let info_log_length_ptr: *mut i32 = &mut info_log_length;
                gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
                // Print any errors that might've happened while compiling this shader source
                if info_log_length > 0 {
                    let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                    gl::GetShaderInfoLog(
                        program,
                        info_log_length,
                        std::ptr::null_mut::<i32>(),
                        log.as_mut_ptr(),
                    );
                    println!("Error while compiling shader source {}!:", source_data.path);
                    let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                    let string = String::from_utf8(printable_log).unwrap();

                    // Put the line count
                    let error_source_lines = source_data.text.lines().enumerate();
                    let error_source = error_source_lines
                        .into_iter()
                        .map(|(count, line)| format!("({}): {}", count + 1, line))
                        .collect::<Vec<String>>()
                        .join("\n");
                    println!("{}", error_source);

                    println!("Error: \n{}", string);
                    panic!();
                }

                println!("Shader Source {} compiled succsessfully!", source_data.path);
                program
            }
        }
        // Extract the shader
        let shader_name = self
            .sources
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>()
            .join("_");

        // Actually compile the shader now
        println!("Compiling & Creating Shader {}...", shader_name);
        let program = unsafe {
            let program = gl::CreateProgram();

            // Create & compile the shader sources and link them
            let taken = std::mem::take(&mut self.sources);
            let programs: Vec<u32> = taken
                .into_iter()
                .map(|(_path, data)| compile_single_source(data))
                .collect::<Vec<_>>();
            // Link
            for shader in programs.iter() {
                gl::AttachShader(program, *shader)
            }

            // Finalize the shader and stuff
            gl::LinkProgram(program);

            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetProgramiv(program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while finalizing this shader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetProgramInfoLog(
                    program,
                    info_log_length,
                    std::ptr::null_mut::<i32>(),
                    log.as_mut_ptr(),
                );
                println!("Error while finalizing shader {}!:", shader_name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n{}", string);
                panic!();
            }
            // Detach shaders
            for shader in programs.iter() {
                gl::DetachShader(program, *shader);
            }
            println!("Shader {} compiled and created succsessfully!", shader_name);
            program
        };
        // Add the shader at the end
        self.program = program;
        // Add the shader
        pipeline.shaders.insert(id, self);
        // And also get it's uniform definition map
        let mappings = query_shader_uniforms_definition_map(program);
        pipeline
            .cached
            .uniform_definitions
            .insert(program, mappings);

        Some(())
    }
    // Remove the shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.shaders.remove(id)
    }
}

impl Shader {
    // Creates a shader from it's corresponding shader settings
    pub fn new(mut settings: ShaderSettings) -> Result<Self, IncludeExpansionError> {
        // Create "self"
        let mut shader = Self {
            program: 0,
            sources: HashMap::default(),
        };
        // Loop through the shader sources and edit them
        let sources = std::mem::take(&mut settings.sources);
        // Loop through all the subshaders and link them
        for (source_path, mut source_data) in sources {
            let mut included_paths: HashSet<String> = HashSet::new();
            // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
            // Include the includables until they cannot be included
            while load_includes(&settings, &mut source_data.text, &mut included_paths)? {
                // We are still including paths
            }
            // Add this shader source to be generated as a subshader
            shader.sources.insert(source_path, source_data);
        }
        Ok(shader)
    }
}
