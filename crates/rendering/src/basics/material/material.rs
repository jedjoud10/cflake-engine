use getset::Getters;

use crate::{
    basics::{shader::Shader, uniforms::Uniforms},
    object::ObjectSealed,
    pipeline::*,
};

// Material that contains a boxed material type
#[derive(Getters)]
pub struct Material {
    // Shader that we will use for the material
    #[getset(get = "pub")]
    shader: Option<Handle<Shader>>,

    // Unique material type for setting the proper uniforms
    boxed: Box<dyn MaterialType>,
}

impl ObjectSealed for Material {
    fn init(&mut self, pipeline: &mut Pipeline) {
        // Get the shader if we don't have it yet
        self.shader.get_or_insert_with(|| self.boxed.shader(pipeline));
    }
}

impl Material {
    // Create a new material with a explicit shader
    pub fn from_parts<M: MaterialType + 'static>(shader: Handle<Shader>, _type: M) -> Self {
        Self {
            shader: Some(shader),
            boxed: Box::new(_type),
        }
    }

    // Create a new material given a material type (we will get the shader when we have access to the pipeline)
    pub fn new<M: MaterialType + 'static>(_type: M) -> Self {
        Self {
            shader: None,
            boxed: Box::new(_type),
        }
    }

    // Set the unique material uniforms
    pub(crate) fn execute(&self, pipeline: &Pipeline, mut uniforms: Uniforms) {
        self.boxed.set(pipeline, &mut uniforms);
    }
}

// Material type trait that will be implement for materials
// TODO: Find a better name for this shit
pub trait MaterialType {
    // Get the default shader for this material type
    fn shader(&self, pipeline: &Pipeline) -> Handle<Shader>; 

    // Write to the proper uniforms
    fn set(&self, pipeline: &Pipeline, uniforms: &mut Uniforms);
}