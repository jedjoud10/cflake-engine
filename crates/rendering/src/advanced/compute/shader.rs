use crate::{
    basics::{
        shader::{load_includes, query_shader_uniforms_definition_map, IncludeExpansionError, ShaderSource, ShaderProgram, ShaderInitSettings},
        uniforms::Uniforms
    },
    pipeline::Pipeline, object::OpenGLInitializer,
};
use ahash::AHashSet;
use getset::Getters;
use gl::types::GLuint;
use std::{collections::HashSet, ffi::CString, ptr::null};

use super::ComputeShaderExecutionSettings;

// A compute shader that can run parallel calculations on the GPU
#[derive(Getters)]
pub struct ComputeShader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    program: ShaderProgram,
    // A single shader source
    #[getset(get = "pub")]
    source: Option<ShaderSource>,

    // Init settings
    #[getset(get = "pub")]
    settings: ShaderInitSettings,
}

impl OpenGLInitializer for ComputeShader {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        // Actually compile the compute shader now
        println!("Compiling & Creating Compute Shader Source {}...", self.source.path);
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
                println!("Error: \n{}", string);
                panic!();
            }

            println!("Subshader {} compiled succsessfully!", self.source.path);
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
                let error_source_lines = self.source.text.lines().enumerate();
                let error_source = error_source_lines
                    .into_iter()
                    .map(|(count, line)| format!("({}): {}", count + 1, line))
                    .collect::<Vec<String>>()
                    .join("\n");
                println!("{}", error_source);
                println!("Error: \n[31m{}", string);
                panic!();
            }
            // Detach shader source
            gl::DetachShader(program, shader_source_program);
            println!("Shader {} compiled and created succsessfully!", self.source.path);
            program
        };
        // Add the shader at the end
        self.program = ShaderProgram {
            program,
            mappings: query_shader_uniforms_definition_map(program),
        };
    }
}

impl ComputeShader {
    // Creates a new compute shader using some shader init settings 
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        let mut included_paths: AHashSet<String> = AHashSet::new();
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
        Ok(Self {
            program: Default::default(),
            source: Default::default(),
            settings,
        })
    }
    /*
    // Run a compute shader, and return it's GlTracker
    pub(crate) fn compute_run(&self, pipeline: &Pipeline, settings: ComputeShaderExecutionSettings) -> GlTracker {
        // Create some shader uniforms settings that we can use
        let uniform_settings = ShaderUniformsSettings::new(ShaderIDType::OpenGLID(self.program));
        let uniforms = Uniforms::new(&uniform_settings, pipeline);
        // Dispatch the compute shader for execution
        let axii = settings.axii;

        // Create the GlTracker and send the DispatchCompute command
        GlTracker::new(|| unsafe {
            uniforms.bind_shader();
            // Execute the uniforms
            for x in settings.callbacks {
                x.execute(&uniforms);
            }
            gl::DispatchCompute(axii.x as u32, axii.y as u32, axii.z as u32);
        })
    }
    */
}
