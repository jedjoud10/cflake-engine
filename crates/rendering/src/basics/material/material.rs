use super::MaterialTextures;
use crate::{basics::shader::Shader, object::PipelineElement, pipeline::*};

// A material that can have multiple parameters and such
pub struct Material {
    // Main settings
    pub shader: Handle<Shader>,

    // Actual parameters used for rendering
    pub textures: MaterialTextures,
    pub tint: vek::Vec3<f32>,
    pub normal_map_strength: f32,
    pub emissive_map_strength: f32,
    pub uv_scale: vek::Vec2<f32>,
}

impl PipelineElement for Material {
    fn add(self, pipeline: &mut Pipeline) -> Handle<Self> {
        pipeline.materials.insert(self)
    }

    fn find<'a>(pipeline: &'a Pipeline, handle: &Handle<Self>) -> Option<&'a Self> {
        pipeline.materials.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut Pipeline, handle: &Handle<Self>) -> Option<&'a mut Self> {
        pipeline.materials.get_mut(handle)
    }

    fn disposed(self) {
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            textures: Default::default(),
            tint: vek::Vec3::one(),
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
            uv_scale: vek::Vec2::one(),
        }
    }
}
