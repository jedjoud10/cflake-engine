use crate::{
    basics::{
        material::{Material, MaterialBuilder},
        shader::Shader,
        texture::Texture2D,
        uniforms::UniformsSet,
    },
    pipeline::Handle,
};


// A physically based material builder that use the PBR shader
pub struct PbrMaterialBuilder {
    // Textures
    diffuse: Handle<Texture2D>,
    normal: Handle<Texture2D>,
    emissive: Handle<Texture2D>,

    // Parameters
    bumpiness: f32,
    emissivity: f32,
    tint: vek::Rgb<f32>,
    scale: vek::Vec2<f32>,
}

impl Default for PbrMaterialBuilder {
    fn default() -> Self {
        Self {
            diffuse: Default::default(),
            normal: Default::default(),
            emissive: Default::default(),
            bumpiness: 1.0,
            emissivity: 0.0,
            tint: vek::Rgb::one(),
            scale: vek::Vec2::one(),
        }
    }
}

impl PbrMaterialBuilder {
    // My eyeballs hurt... again
    pub fn diffuse(mut self, map: Handle<Texture2D>) -> Self {
        self.diffuse = map;
        self
    }

    pub fn normal(mut self, map: Handle<Texture2D>) -> Self {
        self.normal = map;
        self
    }

    pub fn emissive(mut self, map: Handle<Texture2D>) -> Self {
        self.emissive = map;
        self
    }

    // Modifier functions
    pub fn bumpiness(mut self, bumpiness: f32) -> Self {
        self.bumpiness = bumpiness;
        self
    }

    pub fn emissivity(mut self, emissivity: f32) -> Self {
        self.emissivity = emissivity;
        self
    }

    pub fn tint(mut self, tint: vek::Rgb<f32>) -> Self {
        self.tint = tint;
        self
    }

    pub fn scale(mut self, uv_scale: vek::Vec2<f32>) -> Self {
        self.scale = uv_scale;
        self
    }
}

// Convert
impl MaterialBuilder for PbrMaterialBuilder {
    fn build_with_shader(self, pipeline: &mut crate::pipeline::Pipeline, shader: Handle<Shader>) -> Handle<Material> {
        // Clone the default texture handles
        let white = pipeline.defaults().white.clone();
        let black = pipeline.defaults().black.clone();
        let normal_map = pipeline.defaults().normal_map.clone();
        dbg!(white.is_null());
        dbg!(self.diffuse.is_null());

        let mat = Material {
            shader,
            uniforms: UniformsSet::new(move |mut uniforms| {
                // Use default textures if we need to
                let diffuse = self.diffuse.fallback_to(&white);
                let normal = self.normal.fallback_to(&normal_map);
                let emissive = self.emissive.fallback_to(&black);
                uniforms.set_texture2d("diffuse_m", diffuse);
                uniforms.set_texture2d("normal_m", normal);
                uniforms.set_texture2d("emissive_m", emissive);
                // Then the parameters
                uniforms.set_vec3f32("tint", self.tint.into());
                uniforms.set_f32("bumpiness", self.bumpiness);
                uniforms.set_f32("emissivity", self.emissivity);
                uniforms.set_vec2f32("uv_scale", self.scale);
            }),
        };
        pipeline.insert(mat)
    }
}
