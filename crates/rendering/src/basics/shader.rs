use super::SubShader;
use crate::utils::RenderingError;
use crate::SubShaderGPUObject;
use crate::{pipec, GPUObjectID, SubShaderType};
use assets::Object;
use std::collections::{HashMap, HashSet};
// A shader that contains two sub shaders that are compiled independently
#[derive(Clone)]
pub struct Shader {
    pub name: String,
    pub source: String,
    externalcode: HashMap<String, String>,
    pub linked_subshaders: Vec<GPUObjectID>,
}

impl Default for Shader {
    fn default() -> Self {
        Self {
            name: String::new(),
            source: String::new(),
            externalcode: HashMap::new(),
            linked_subshaders: Vec::new(),
        }
    }
}

// A shader is an asset object, while a subshader is an asset
impl Object for Shader {
    fn get_unique_object_name(&self, _local_path: &str) -> String {
        self.name.to_string()
    }
}

impl Shader {
    // Load the files that need to be included for this specific shader and return the included lines
    fn load_includes<'a>(&self, subshader_name: &str, lines: &mut Vec<String>, included_paths: &mut HashSet<String>) -> Result<bool, RenderingError> {
        let mut vectors_to_insert: Vec<(usize, Vec<String>)> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            // Check if this is an include statement
            if line.starts_with("#include ") {
                // Get the local path of the include file
                let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace('"', "");
                let local_path = local_path.trim_start();
                if !included_paths.contains(&local_path.to_string()) {
                    // Load the function shader text
                    included_paths.insert(local_path.to_string());
                    let text = assets::assetc::load_text(local_path).map_err(|_| {
                        RenderingError::new(format!(
                            "Tried to include function shader '{}' and it was not pre-loaded!. Shader '{}'",
                            local_path, subshader_name
                        ))
                    })?;
                    let new_lines = text.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                    vectors_to_insert.push((i, new_lines));
                }
            }
            // External shader code
            if !self.externalcode.is_empty() && line.trim().starts_with("#include_custom") {
                // Get the source
                let c = line.split("#include_custom ").collect::<Vec<&str>>()[1];
                let source_id = &c[2..(c.len() - 2)];
                let source = self.externalcode.get(source_id).unwrap();
                let lines = source.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                vectors_to_insert.push((i, lines));
            }
            // Impl default types
            if line.trim().starts_with("#load_defaults") {
                let x = match line.split("#load_defaults ").collect::<Vec<&str>>()[1] {
                    "renderer" => {
                        vectors_to_insert.push((i, vec!["#include defaults\\shaders\\others\\default_impls\\renderer.func.glsl".to_string()]));
                        Ok(())
                    }
                    x => Err(RenderingError::new(format!("Tried to expand #load_defaults, but the given type '{}' is not valid!", x))),
                };
                x?;
            }
        }
        // Add the newly included lines at their respective index
        let mut offset = 0;
        for (i, _) in vectors_to_insert.iter() {
            let x = lines.get_mut(*i).unwrap();
            *x = String::default();
        }
        for (i, included_lines) in vectors_to_insert.iter() {
            // Remove the include
            for x in 0..included_lines.len() {
                let new_index = x + offset + *i;
                lines.insert(new_index, included_lines[x].clone());
            }
            // Add the offset so the next lines will be at their correct positions
            offset += included_lines.len();
        }
        Ok(!vectors_to_insert.is_empty())
    }
    // Creates a shader from multiple subshader files
    pub fn load_shader(mut self, subshader_paths: Vec<&str>) -> Result<Self, RenderingError> {
        // Create the shader name
        self.name = format!("shader_{}", subshader_paths.join("__"));
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through all the subshaders and link them
        for subshader_path in subshader_paths {
            // Check if we even have the subshader cached (In the object cacher) and check if it's cached in the pipeline as well
            if assets::cachec::cached(subshader_path) && pipec::others::gpuobject_name_valid(subshader_path) {
                let id = pipec::others::get_id(subshader_path).unwrap();
                self.linked_subshaders.push(id);
            } else {
                // It was not cached, so we need to cache it
                let mut subshader: SubShader = assets::assetc::dload(subshader_path).map_err(|_| RenderingError::new_str("Sub-shader was not pre-loaded!"))?;
                // Recursively load the shader includes
                let lines = subshader.source.lines().collect::<Vec<&str>>();
                let lines = lines.clone().iter().map(|x| x.to_string()).collect::<Vec<String>>();
                // Included lines
                let mut included_lines: Vec<String> = lines;
                // Include the sources until no sources can be included
                while Self::load_includes(&self, subshader_path, &mut included_lines, &mut included_paths)? {
                    // We are still including paths
                }
                // Set the shader source for this shader
                let extend_shader_source = included_lines.join("\n");

                // Remove the version directive from the original subshader source
                subshader.source = extend_shader_source;
                // Gotta filter out the include messages
                subshader.source = subshader
                    .source
                    .lines()
                    .filter(|x| {
                        let s = x.to_string();
                        let s = s.trim();
                        !s.starts_with("#include") && !s.starts_with("#include_custom")
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                // Cache it, and link it
                self.linked_subshaders.push(pipec::subshader(subshader.clone()));
                let _rc_subshader = assets::cachec::cache_l(subshader_path, subshader).unwrap();
            }
        }
        Ok(self)
    }
    // Load some external code that can be loading using specific include points
    pub fn load_externalcode(mut self, id: &str, string: String) -> Self {
        self.externalcode.insert(id.to_string(), string);
        self
    }
}

// Each shader can be instanced
impl others::Instance for Shader {
    fn set_name(&mut self, string: String) {
        self.name = string
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}
