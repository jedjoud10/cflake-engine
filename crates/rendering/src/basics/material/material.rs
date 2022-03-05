use getset::{Getters, MutGetters, Setters};

use crate::{pipeline::*, basics::{uniforms::SetUniformsCallback, texture::Texture, shader::Shader}, object::PipelineCollectionElement};

// Material textures
#[derive(Default, Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct MaterialTextures {
    diffuse_map: Handle<Texture>,
    normal_map: Handle<Texture>,
    emissive_map: Handle<Texture>,
}

// A material that can have multiple parameters and such
#[derive(Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Material {
    // Main settings
    shader: Handle<Shader>,
    uniforms: SetUniformsCallback,

    // Actual parameters used for rendering
    textures: MaterialTextures,
    tint: veclib::Vector3<f32>,
    normal_map_strength: f32,
    emissive_map_strength: f32,
    uv_scale: veclib::Vector2<f32>,
}

impl PipelineCollectionElement for Material {
    fn added(&mut self, collection: &mut PipelineCollection<Self>, handle: Handle<Self>) {
    }

    fn disposed(self) {
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: Default::default(),
            textures: Default::default(),
            tint: veclib::Vector3::ONE,
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
            uv_scale: veclib::Vector2::ONE,
        }
    }
}