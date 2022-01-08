use crate::object::{PipelineObject, PipelineTask, ObjectBuildingTask};
use crate::utils::RenderingError;
use crate::{pipec, object::ObjectID};
use crate::{params::*, Buildable};
use std::collections::{HashSet, HashMap};
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
    // Some external code that we can 
    pub(crate) external_code: HashMap<u8, String>,
    pub(crate) sources: HashMap<String, ShaderSource>,
}

impl ShaderSettings {
    // Load some external code that can be loading using specific include points
    pub fn external_code(mut self, id: u8, string: String) -> Self {
        self.external_code.insert(id, string);
        self
    }
    // Load a shader source
    pub fn source(mut self, path: &str) -> Self {        
        // Load a shader source from scratch
        let metadata = assets::assetc::raw_metadata(path).unwrap();
        let text = assets::assetc::load_text(path).unwrap();
        self.sources.insert(path.to_string(), ShaderSource {
            path: path.to_string(),
            text,
            _type: match metadata.extension {
                "vrsh.glsl" => ShaderSourceType::Vertex,
                "frsh.glsl" => ShaderSourceType::Fragment,
                "cmpt.glsl" => ShaderSourceType::Compute,
                _ => panic!()
            },
        });
        self
    }
}

// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
pub struct Shader {
    // The updated and modified shader sources
    pub(crate) sources: HashMap<String, ShaderSource>,
}
impl PipelineObject for Shader {}

impl Buildable for Shader {
    fn construct(self, pipeline: &crate::Pipeline) -> ObjectID<Self> {
        // Create the ID
        let id = pipeline.shaders.get_next_idx_increment();
        let id = ObjectID::new(id);
        crate::pipec::task(PipelineTask::CreateShader(ObjectBuildingTask::<Self>(self, id)), pipeline);
        id
    }

    fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
}


impl Shader {
    // Load the files that need to be included for this specific shader and return the included lines
    fn load_includes<'a>(&self, subshader_name: &str, source: &mut String, included_paths: &mut HashSet<String>) -> Result<bool, RenderingError> {
        // Turn the string into lines
        let mut lines = source.lines().into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
        for (i, line) in lines.iter_mut().enumerate() {
            // Check if this is an include statement
            if line.starts_with("#include ") {
                // Get the local path of the include file
                let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace('"', "");
                let local_path = local_path.trim_start();
                
                // Load the include function text
                let text = if !included_paths.contains(&local_path.to_string()) {
                    // Load the function shader text
                    included_paths.insert(local_path.to_string());
                    assets::assetc::load_text(local_path).map_err(|_| {
                        RenderingError::new(format!(
                            "Tried to include function shader '{}' and it was not pre-loaded!. Shader '{}'",
                            local_path, subshader_name
                        ))
                    })?
                } else { String::new() };

                // Update the original line
                *line = text; 
                break;
            }
            // External shader code
            if !self.externalcode.is_empty() && line.trim().starts_with("#include_custom ") {
                // Get the source
                let c = line.split("#include_custom ").collect::<Vec<&str>>()[1];
                let source_id = &c[2..(c.len() - 2)];
                let source = self.externalcode.get(source_id).unwrap();
                *line = source.clone();
                break;
            }
            // Impl default types
            if line.trim().starts_with("#load") {
                let x = match line.split("#load ").collect::<Vec<&str>>()[1] {
                    "renderer" => {
                        *line = "#include defaults\\shaders\\others\\default_impls\\renderer.func.glsl".to_string();
                        Ok(())
                    }
                    "renderer_main_start" => {
                        *line = "#include defaults\\shaders\\others\\default_impls\\renderer_main_start.func.glsl".to_string();          
                        Ok(())            
                    }
                    "renderer_life_fade" => {
                        *line = "#include defaults\\shaders\\others\\default_impls\\renderer_life_fade.func.glsl".to_string();
                        Ok(())                        
                    }
                    x => Err(RenderingError::new(format!("Tried to expand #load, but the given type '{}' is not valid!", x))),
                };
                x?;
                break;
            }
            // Constants
            if line.trim().contains("#constant ") {
                fn format(line: &String, val: String) -> String {
                    format!("{} {};", line.trim().split("#constant").nth(0).unwrap(), val)
                }
                let x = match line.split("#constant ").collect::<Vec<&str>>()[1] {
                    "fade_in_speed" => {
                        *line = format(line, FADE_IN_SPEED.to_string());   
                        Ok(())
                    }
                    "fade_out_speed" => {
                        *line = format(line, FADE_OUT_SPEED.to_string());          
                        Ok(())            
                    }
                    x => Err(RenderingError::new(format!("Tried to expand #constant, but the given type '{}' is not valid!", x))),
                };
                x?;
                break;
            }
        }
        // Update the source
        *source = lines.join("\n");
        // Check if we need to continue expanding the includes        
        let need_to_continue = lines.iter().any(|x| x.trim().starts_with("#include ") || x.trim().starts_with("#include_custom ") || x.trim().starts_with("#load ") || x.trim().contains("#constant "));
        Ok(need_to_continue)
    }
    // Creates a shader from it's corresponding shader settings
    pub fn load_shader(mut self, settings: ShaderSettings) -> Result<Self, RenderingError> {
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
                let mut final_source = subshader.source;
                // Include the sources until no sources can be included
                while Self::load_includes(&self, subshader_path, &mut final_source, &mut included_paths)? {
                    // We are still including paths
                }
                // Set the shader source for this shader
                subshader.source = final_source;
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
}
