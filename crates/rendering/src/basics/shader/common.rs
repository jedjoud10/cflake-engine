// Some shared code between the normal shaders and the compute shaders

use std::{
    collections::HashSet,
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr::{null, null_mut}, cell::RefCell,
};

use ahash::{AHashMap, AHashSet};
use enum_as_inner::EnumAsInner;
use getset::{CopyGetters, Getters, MutGetters};
use gl::types::GLuint;

use crate::{basics::shader::ShaderSourceType, object::OpenGLObjectNotInitialized, pipeline::Pipeline};

use super::{
    info::{QueryParameter, QueryResource, Resource, ShaderInfo, ShaderInfoQuerySettings, UpdatedParameter},
    IncludeExpansionError, ShaderSource,
};

// Uniforms definition map
pub type UniformsDefinitionMap = AHashMap<String, i32>;


// Used texture units
pub type UsedTextureUnits = RefCell<AHashMap<String, usize>>;

// Load the files that need to be included for this specific shader and return the included lines
pub(crate) fn load_includes(settings: &ShaderInitSettings, source: &mut String, included_paths: &mut AHashSet<String>) -> Result<bool, IncludeExpansionError> {
    // Turn the string into lines
    let mut lines = source.lines().into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
    for (_i, line) in lines.iter_mut().enumerate() {
        // Check if this is an include statement
        if line.starts_with("#include ") {
            // Get the local path of the include file
            let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace('"', "");
            let local_path = local_path.trim_start();

            // Load the include function text
            let text = if !included_paths.contains(&local_path.to_string()) {
                // Load the function shader text
                included_paths.insert(local_path.to_string());
                assets::assetc::load(local_path)
                    .map_err(|_| IncludeExpansionError::new(format!("Tried to include function shader '{}' and it was not pre-loaded!.", local_path)))?
            } else {
                String::new()
            };

            // Update the original line
            *line = text;
            break;
        }
        // External shader code
        if line.trim().starts_with("#include_custom ") {
            // Get the source
            let c = line.split("#include_custom ").collect::<Vec<&str>>()[1];
            let source_name = &c[2..(c.len() - 2)].to_string();
            let source = settings
                .directives()
                .get(source_name)
                .map(|directive| directive.as_external())
                .flatten()
                .ok_or(IncludeExpansionError::new(format!(
                    "Tried to expand #include_custom, but the given source name '{}' is not valid!",
                    source_name
                )))?;
            *line = source.clone();
            break;
        }
        // Impl default types
        if line.trim().starts_with("#load") {
            let x = match line.split("#load ").collect::<Vec<&str>>()[1] {
                // Refactor this
                "renderer" => {
                    *line = "#include defaults/shaders/others/default_impls/renderer.func.glsl".to_string();
                    Ok(())
                }
                "general" => {
                    *line = "#include defaults/shaders/others/default_impls/general.func.glsl".to_string();
                    Ok(())
                }
                x => Err(IncludeExpansionError::new(format!("Tried to expand #load, but the given type '{}' is not valid!", x))),
            };
            x?;
            break;
        }
        // Constants
        if line.trim().contains("#constant ") {
            fn format(line: &str, val: &str) -> String {
                format!("{}{};", line.trim().split("#constant").next().unwrap(), val)
            }
            // Expand
            let const_name = line.split("#constant ").collect::<Vec<&str>>()[1];
            let _const_line = settings
                .directives()
                .get(const_name)
                .map(|directive| Some(format(line, directive.as_const()?.as_str())))
                .flatten();
            *line = _const_line.ok_or(IncludeExpansionError::new(format!(
                "Tried to expand #constant, but the given const name '{}' is not valid!",
                const_name
            )))?;
            break;
        }
    }
    // Update the source
    *source = lines.join("\n");
    // Check if we need to continue expanding the includes
    let need_to_continue = lines
        .iter()
        .any(|x| x.trim().starts_with("#include ") || x.trim().starts_with("#include_custom ") || x.trim().starts_with("#load ") || x.trim().contains("#constant "));
    Ok(need_to_continue)
}

// Compile a whole shader using it's sources
pub(crate) fn compile_shader(sources: &AHashMap<String, ShaderSource>) -> ShaderProgram {
    // Loop through all the subshaders and link them
    // Extract the shader name
    let shader_name = sources.iter().map(|(name, _)| name.clone()).collect::<Vec<String>>().join("_");

    // Actually compile the shader now
    println!("Compiling & Creating Shader {}...", shader_name);
    let program = unsafe {
        let program = gl::CreateProgram();

        // Create & compile the shader sources and link them
        let programs: Vec<u32> = sources.iter().map(|(_path, data)| compile_source(data)).collect::<Vec<_>>();
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
            gl::GetProgramInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
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
    ShaderProgram {
        program: program,
        mappings: query_shader_uniforms_definition_map(program),
        used_texture_units: Default::default(),
    }
}

// Compile a shader source
pub(crate) fn compile_source(source: &ShaderSource) -> GLuint {
    println!("Compiling & Creating Shader Source {}...", source.file());
    let shader_type: u32 = match source._type() {
        ShaderSourceType::Vertex => gl::VERTEX_SHADER,
        ShaderSourceType::Fragment => gl::FRAGMENT_SHADER,
        ShaderSourceType::Compute => gl::COMPUTE_SHADER,
    };
    unsafe {
        // Compiling the source
        let program = gl::CreateShader(shader_type);

        // Compile the shader
        let cstring = CString::new(source.text().clone()).unwrap();
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
            println!("Error while compiling shader source {} of type {:?}!:", source.file(), source._type());
            let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
            let string = String::from_utf8(printable_log).unwrap();

            // Put the line count
            let error_source_lines = source.text().lines().enumerate();
            let error_source = error_source_lines
                .into_iter()
                .map(|(count, line)| format!("({}): {}", count + 1, line))
                .collect::<Vec<String>>()
                .join("\n");

            println!("{}", error_source);
            println!("Error: \n{}", string);
            panic!();
        }

        println!("Shader Source {} compiled succsessfully!", source.file());
        program
    }
}

// Get the uniform definition map from a shader using the query API
pub(crate) fn query_shader_uniforms_definition_map(program: u32) -> UniformsDefinitionMap {
    let mut settings = ShaderInfoQuerySettings::default();
    settings.query_all(QueryResource::Uniform, vec![QueryParameter::Location]);
    let res = query_shader_info(program, settings);
    let maybe = res.get_all(&QueryResource::Uniform);
    // In some cases we might not have uniforms at all
    if let Some(mappings) = maybe {
        let mappings = mappings
            .iter()
            .filter_map(|(name, params)| {
                // Get the inner location
                let location = *params.get(0)?.as_location()? as i32;
                Some((name.clone(), location))
            })
            .collect::<AHashMap<_, _>>();
        mappings
    } else {
        UniformsDefinitionMap::default()
    }
}

// Query some information about a shader, and then return
pub(crate) fn query_shader_info(program: GLuint, settings: ShaderInfoQuerySettings) -> ShaderInfo {
    unsafe {
        // Get the query info
        // Gotta count the number of unique resource types
        let mut unique_count = AHashMap::<QueryResource, usize>::new();
        let mut indexed_resources = AHashMap::<Resource, (Vec<QueryParameter>, usize)>::new();
        for (x, parameters) in settings.res.iter() {
            let count = unique_count.entry(x.res.clone()).or_default();
            indexed_resources.insert(x.clone(), (parameters.clone(), *count));
            *count += 1;
        }
        // Also do the same for the unique ALL resources
        // This only adds to the unique_count, and doesn't modify indexed_resources
        for (x, _parameters) in settings.res_all.iter() {
            let count = unique_count.entry(x.clone()).or_default();
            *count += 1;
        }

        // First we gotta get how many resources of a single type we have, and their respective max name len
        let types_and_counts = unique_count
            .iter()
            .map(|(res, _)| {
                let mut max_resources = 0_i32;
                let mut max_name_len = 0_i32;
                gl::GetProgramInterfaceiv(program, res.convert(), gl::ACTIVE_RESOURCES, &mut max_resources);
                gl::GetProgramInterfaceiv(program, res.convert(), gl::MAX_NAME_LENGTH, &mut max_name_len);
                (res.clone(), (max_resources, max_name_len as usize))
            })
            .collect::<AHashMap<_, _>>();

        // Now we can actually query the parameters
        let mut output_queried_resources = AHashMap::<Resource, Vec<UpdatedParameter>>::new();
        // Get the specific parameters
        for (res, (parameters, _i)) in indexed_resources {
            let cstring = CString::new(res.name.clone()).unwrap();
            // Get the resource's index
            let resource_index = gl::GetProgramResourceIndex(program, res.convert(), cstring.as_ptr());
            if resource_index == gl::INVALID_INDEX {
                panic!()
            }

            // Now we can finally access the resource's parameters
            let converted_params = parameters.iter().map(|x| x.convert()).collect::<Vec<_>>();
            let max_len = converted_params.len();
            let mut output = vec![-1; max_len];
            gl::GetProgramResourceiv(
                program,
                res.convert(),
                resource_index,
                max_len as i32,
                converted_params.as_ptr(),
                output.len() as i32,
                null_mut(),
                output.as_mut_ptr(),
            );

            // Check for negative numbers, and remove them
            let output = output.into_iter().filter(|&x| x != -1).collect::<Vec<_>>();

            let converted_outputs = parameters
                .iter()
                .zip(output)
                .map(|(x, opengl_val)| x.convert_output(opengl_val))
                .collect::<Vec<UpdatedParameter>>();

            // After reading everything, we can add convert the output values into their respective Rust enums and store them
            output_queried_resources.insert(res, converted_outputs);
        }
        // Get ALL the parameters, if we want to
        let mut output_queried_resources_all = AHashMap::<QueryResource, Vec<(String, Vec<UpdatedParameter>)>>::new();
        for (unique_resource, (max_resources, max_name_len)) in types_and_counts.into_iter() {
            // No need
            if !settings.res_all.contains_key(&unique_resource) {
                continue;
            }
            for id in 0..max_resources {
                // Get the resource's parameters first
                let unique_resource_parameters = settings.res_all.get(&unique_resource).unwrap();
                let gl_parameters = unique_resource_parameters.iter().map(|x| x.convert()).collect::<Vec<_>>();
                let max_len = unique_resource_parameters.len();
                let mut output = vec![-1; max_len];
                if (id as u32) == gl::INVALID_INDEX {
                    panic!()
                }
                gl::GetProgramResourceiv(
                    program,
                    unique_resource.convert(),
                    id as u32,
                    max_len as i32,
                    gl_parameters.as_ptr(),
                    output.len() as i32,
                    null_mut(),
                    output.as_mut_ptr(),
                );

                // Check for negative numbers, and remove them
                let output = output.into_iter().filter(|&x| x != -1).collect::<Vec<_>>();

                let mut name = vec![c_char::default(); max_name_len + 1];
                // Get the resource's name
                gl::GetProgramResourceName(program, unique_resource.convert(), id as u32, name.len() as i32, null_mut(), name.as_mut_ptr());
                let name = CStr::from_ptr(name.as_ptr()).to_str().unwrap().to_string();
                let converted_outputs = unique_resource_parameters
                    .iter()
                    .zip(output)
                    .map(|(x, opengl_val)| x.convert_output(opengl_val))
                    .collect::<Vec<UpdatedParameter>>();
                let entry = output_queried_resources_all.entry(unique_resource.clone()).or_default();
                entry.push((name, converted_outputs));
            }
        }
        ShaderInfo {
            res: output_queried_resources,
            res_all: output_queried_resources_all,
        }
    }
}

// Querry shader info
pub fn query_info(program: &ShaderProgram, _pipeline: &Pipeline, settings: ShaderInfoQuerySettings) -> Result<ShaderInfo, OpenGLObjectNotInitialized> {
    // Check validity
    if program.program() == 0 {
        return Err(OpenGLObjectNotInitialized);
    }
    Ok(query_shader_info(program.program(), settings))
}

// A single shader directive
#[derive(EnumAsInner)]
pub enum Directive {
    Const(String),
    External(String),
}

// Shader init settings (sources, additional code, consts)
#[derive(Getters, MutGetters, Default)]
pub struct ShaderInitSettings {
    #[getset(get = "pub")]
    directives: AHashMap<String, Directive>,
    #[getset(get = "pub", get_mut = "pub(crate)")]
    sources: AHashMap<String, ShaderSource>,
}

impl ShaderInitSettings {
    // Add a shader directive
    pub fn directive(mut self, name: &str, directive: Directive) -> Self {
        self.directives.insert(name.to_string(), directive);
        self
    }
    // Add a source
    pub fn source(mut self, path: &str) -> Self {
        self.sources.insert(path.to_string(), assets::assetc::load(path).unwrap());
        self
    }
}

// A shader program that contains an OpenGL program ID and the shader definition map
#[derive(Getters, CopyGetters, Default, Clone)]
pub struct ShaderProgram {
    #[getset(get_copy = "pub")]
    program: GLuint,
    #[getset(get = "pub")]
    mappings: UniformsDefinitionMap,
    #[getset(get = "pub(crate)")]
    used_texture_units: UsedTextureUnits,
}
