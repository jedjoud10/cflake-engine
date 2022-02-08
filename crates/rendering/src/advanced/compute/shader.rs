use std::{collections::HashSet, ffi::CString, ptr::null};

use crate::{
    basics::{
        shader::{load_includes, query_shader_uniforms_definition_map, IncludeExpansionError, ShaderSettings, ShaderSource},
        uniforms::{ShaderIDType, ShaderUniformsSettings, Uniforms},
    },
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID, PipelineObject},
    pipeline::Pipeline,
};

use super::ComputeShaderExecutionSettings;

// A compute shader that can run parallel calculations on the GPU
pub struct ComputeShader {
    // The OpenGL program linked to this compute shader
    pub(crate) program: u32,
    // We only have one shader source since we are a compute shader
    pub(crate) source: ShaderSource,
}
impl PipelineObject for ComputeShader {
    // Reserve an ID for this compute shader
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.compute_shaders.gen_id()))
    }
    // Send this compute shader to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::ComputeShader(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::ComputeShader(Deconstruct::<Self>(id))
    }
    // Add the compute shader to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Actually compile the compute shader now
        println!("\x1b[33mCompiling & Creating Compute Shader Source {}...\x1b[0m", self.source.path);
        let shader_source_program = unsafe {
            // Compiling the source
            let program = gl::CreateShader(gl::COMPUTE_SHADER);
            // Compile the shader
            let cstring = CString::new(self.source.text.clone()).unwrap();
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
                gl::GetShaderInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while compiling shader source {}!:", self.source.path);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                // Put the line count
                let error_source_lines = self.source.text.lines().enumerate();
                let error_source = error_source_lines
                    .into_iter()
                    .map(|(count, line)| format!("({}): {}", count + 1, line))
                    .collect::<Vec<String>>()
                    .join("\n");
                println!("{}", error_source);
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }

            println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", self.source.path);
            program
        };
        let program = unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, shader_source_program);
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
                gl::GetProgramInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while finalizing shader {}!:", self.source.path);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }
            // Detach shader source
            gl::DetachShader(program, shader_source_program);
            println!("\x1b[32mShader {} compiled and created succsessfully!\x1b[0m", self.source.path);
            program
        };
        // Add the shader at the end
        self.program = program;
        // Add the compute shader
        pipeline.compute_shaders.insert(id, self);
        // And also get it's uniform definition map
        if let Some(mappings) = query_shader_uniforms_definition_map(program) { pipeline.cached.uniform_definitions.insert(program, mappings); }        
        Some(())
    }
    // Remove the compute shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.compute_shaders.remove(id)
    }
}

impl ComputeShader {
    // Creates a compute shader from it's corresponding shader settings
    pub fn new(mut settings: ShaderSettings) -> Result<Self, IncludeExpansionError> {
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(&mut settings.sources);
        // Since this is a compute shader, we only have one source
        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        let (_, mut source_data) = sources.drain().collect::<Vec<_>>().remove(0);
        while load_includes(&settings, &mut source_data.text, &mut included_paths)? {
            // We are still including paths
        }
        // Add this shader source to be generated as a subshader
        Ok(Self { program: 0, source: source_data })
    }
    // Run a compute shader, and return it's GlTracker
    pub(crate) fn compute_run(&self, pipeline: &Pipeline, settings: ComputeShaderExecutionSettings) -> GlTracker {
        // Create some shader uniforms settings that we can use
        let uniform_settings = ShaderUniformsSettings::new(ShaderIDType::OpenGLID(self.program));
        let uniforms = Uniforms::new(&uniform_settings, pipeline);
        // Dispatch the compute shader for execution
        let axii = settings.axii;

        // Create the GlTracker and send the DispatchCompute command
        GlTracker::new(
            |_| unsafe {
                uniforms.bind_shader();
                settings.callback.execute(&uniforms);
                gl::DispatchCompute(axii.0 as u32, axii.1 as u32, axii.2 as u32);
            },
            |_| {},
            pipeline,
        )
    }
}
