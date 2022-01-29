use crate::basics::Buildable;
use crate::object::ObjectID;
use crate::object::{ObjectBuildingTask, PipelineObject, PipelineTask};
use crate::params::{FADE_IN_SPEED, FADE_OUT_SPEED};
use crate::pipeline::Pipeline;
use crate::utils::RenderingError;
use bitflags::bitflags;
use std::collections::{HashMap, HashSet};

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
        let mut consts = HashMap::new();
        consts.insert("fade_in_speed".to_string(), FADE_IN_SPEED.to_string());
        consts.insert("fade_out_speed".to_string(), FADE_OUT_SPEED.to_string());

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
        let metadata = assets::assetc::raw_metadata(path).unwrap();
        let text = assets::assetc::load_text(path).unwrap();
        self.sources.insert(
            path.to_string(),
            ShaderSource {
                path: path.to_string(),
                text,
                _type: match metadata.extension.as_str() {
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
    pub(crate) program: u32,
    // The updated and modified shader sources
    pub(crate) sources: HashMap<String, ShaderSource>,
}
impl PipelineObject for Shader {}

impl Buildable for Shader {
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.shaders.get_next_id_increment();
        let id = ObjectID::new(id);
        (PipelineTask::CreateShader(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

// Load the files that need to be included for this specific shader and return the included lines
pub(crate) fn load_includes(settings: &ShaderSettings, source: &mut String, included_paths: &mut HashSet<String>) -> Result<bool, RenderingError> {
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
                assets::assetc::load_text(local_path).map_err(|_| RenderingError::new(format!("Tried to include function shader '{}' and it was not pre-loaded!.", local_path)))?
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
            let source = settings.external_code.get(source_name).expect(&format!("Tried to expand #include_custom, but the given source name '{}' is not valid!", source_name));
            *line = source.clone();
            break;
        }
        // Impl default types
        if line.trim().starts_with("#load") {
            let x = match line.split("#load ").collect::<Vec<&str>>()[1] {
                // Refactor this
                "renderer" => {
                    *line = "#include defaults\\shaders\\others\\default_impls\\renderer.func.glsl".to_string();
                    Ok(())
                }
                "general" => {
                    *line = "#include defaults\\shaders\\others\\default_impls\\general.func.glsl".to_string();
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
            fn format(line: &String, val: &String) -> String {
                format!("{}{};", line.trim().split("#constant").next().unwrap(), val)
            }
            let const_name = line.split("#constant ").collect::<Vec<&str>>()[1];
            let x = settings.consts.get(const_name);
            if let Some(x) = x {
                *line = format(line, x);
                Ok(())
            } else {
                Err(RenderingError::new(format!(
                    "Tried to expand #constant, but the given const name '{}' is not valid!",
                    const_name
                )))
            }?;
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

impl Shader {
    // Creates a shader from it's corresponding shader settings
    pub fn new(mut settings: ShaderSettings) -> Result<Self, RenderingError> {
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
