use crate::utils::RenderingError;
use crate::GPUObject;

use super::SubShader;
use super::SubShaderType;
use super::Texture;
use super::TextureShaderAccessType;
use assets::Asset;
use assets::AssetManager;
use assets::Object;
use gl;

use std::collections::HashSet;
use std::ffi::CString;
use std::rc::Rc;

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
    pub name: String,
    pub source: String,
    pub linked_subshaders_programs: Vec<GPUObject>,
}

impl Default for Shader {
    fn default() -> Self {
        unsafe {
            Self {
                name: "".to_string(),
                source: "".to_string(),
                linked_subshaders_programs: Vec::new(),
            }
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
    fn load_includes<'a>(
        &self,
        subshader_name: &str,
        lines: &mut Vec<String>,
        asset_manager: &'a AssetManager,
        included_paths: &mut HashSet<String>,
    ) -> Result<bool, RenderingError> {
        let mut vectors_to_insert: Vec<(usize, Vec<String>)> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            // Check if this is an include statement
            if line.starts_with("#include ") {
                // Get the local path of the include file
                let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace(r#"""#, "");
                let local_path = local_path.trim_start();
                if !included_paths.contains(&local_path.to_string()) {
                    // Load the function shader text
                    included_paths.insert(local_path.to_string());
                    let text = asset_manager.asset_cacher.load_text(local_path).map_err(|_x| {
                        RenderingError::new(format!(
                            "Tried to include function shader '{}' and it was not pre-loaded!. Shader '{}'",
                            local_path, subshader_name
                        ))
                    })?;
                    let new_lines = text.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                    vectors_to_insert.push((i, new_lines));
                }
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
    pub fn load_shader<'a>(mut self, subshader_paths: Vec<&str>, asset_manager: &'a mut AssetManager) -> Result<Self, RenderingError> {
        // Create the shader name
        self.name = subshader_paths.join("__");
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through all the subshaders and link them
        for subshader_path in subshader_paths {
            // Check if we even have the subshader cached
            if asset_manager.object_cacher.cached(subshader_path) {
                let rc_subshader = SubShader::object_load_o(subshader_path, &asset_manager.object_cacher);
                let subshader = rc_subshader.as_ref();
                self.link_subshader(subshader);
            } else {
                // It was not cached, so we need to cache it
                let mut subshader = SubShader::default()
                    .load_asset(subshader_path, &asset_manager.asset_cacher)
                    .ok_or(RenderingError::new_str("Sub-shader was not pre-loaded!"))?;
                // Recursively load the shader includes
                let lines = subshader.source.lines().collect::<Vec<&str>>();
                let lines = lines.clone().iter().map(|x| x.to_string()).collect::<Vec<String>>();
                // Included lines
                let mut included_lines: Vec<String> = lines;
                // Include the sources until no sources can be included
                while Self::load_includes(&self, subshader_path, &mut included_lines, asset_manager, &mut included_paths)? {
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
                // Compile the subshader
                subshader.compile_subshader();

                // Cache it, and link it
                let rc_subshader: Rc<SubShader> = asset_manager.object_cacher.cache(subshader_path, subshader).unwrap();
                self.link_subshader(rc_subshader.as_ref());
            }
        }
        // Finalize the shader and cache it
        self.finalize_shader();
        Ok(self)
    }
    // Cache this shader
    pub fn cache<'a>(self, asset_manager: &'a mut AssetManager) -> Rc<Self> {
        let name = self.name.clone();
        asset_manager.object_cacher.cache(&name, self).unwrap()
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
