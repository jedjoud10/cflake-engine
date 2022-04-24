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
    
    // R: AO
    // G: Roughness
    // B: Metallic
    mask: Handle<Texture2D>,

    // Parameters
    bumpiness: f32,
    emissivity: f32,
    tint: vek::Rgb<f32>,
    scale: vek::Vec2<f32>,
    ao: f32,
    roughness: f32,
    metallic: f32,
}

impl Default for PbrMaterialBuilder {
    fn default() -> Self {
        Self {
            diffuse: Default::default(),
            normal: Default::default(),
            emissive: Default::default(),
            mask: Default::default(),
            bumpiness: 1.0,
            emissivity: 0.0,
            tint: vek::Rgb::one(),
            scale: vek::Vec2::one(),
            ao: 1.0,
            roughness: 1.0,
            metallic: 1.0,
        }
    }
}

impl PbrMaterialBuilder {
    // Set the diffuse texture
    pub fn diffuse(mut self, map: Handle<Texture2D>) -> Self {
        self.diffuse = map;
        self
    }

    // Set the normal map texture
    pub fn normal(mut self, map: Handle<Texture2D>) -> Self {
        self.normal = map;
        self
    }

    // Set the emissive texture
    pub fn emissive(mut self, map: Handle<Texture2D>) -> Self {
        self.emissive = map;
        self
    }

    // Set the mask texture; A texture that contains AO/Roughness/Metallic in each channel
    pub fn mask(mut self, mask: Handle<Texture2D>) -> Self {
        self.mask = mask;
        self
    }

    // Update the normal map's strength
    pub fn bumpiness(mut self, bumpiness: f32) -> Self {
        self.bumpiness = bumpiness;
        self
    }

    // Global emissive strength
    pub fn emissivity(mut self, emissivity: f32) -> Self {
        self.emissivity = emissivity;
        self
    }

    // Roughness factor
    pub fn roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    // Metallic factor
    pub fn metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    // Global AO strength
    pub fn ao_factor(mut self, strength: f32) -> Self {
        self.ao = strength;
        self
    }

    // Main color tint of the material
    pub fn tint(mut self, tint: vek::Rgb<f32>) -> Self {
        self.tint = tint;
        self
    }

    // UV scale of the material
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
        let mask = pipeline.defaults().mask.clone();
        let normal_map = pipeline.defaults().normal_map.clone();

        let mat = Material {
            shader,
            uniforms: UniformsSet::new(move |mut uniforms| {
                // Use default textures if we need to
                let diffuse = self.diffuse.fallback_to(&white);
                let normal = self.normal.fallback_to(&normal_map);
                let emissive = self.emissive.fallback_to(&black);
                let mask = self.mask.fallback_to(&mask);
                uniforms.set_texture2d("diffuse_m", diffuse);
                uniforms.set_texture2d("normal_m", normal);
                uniforms.set_texture2d("emissive_m", emissive);
                uniforms.set_texture2d("mask_m", mask);
                // Then the parameters
                uniforms.set_vec3f32("tint", self.tint.into());
                uniforms.set_f32("bumpiness", self.bumpiness);
                uniforms.set_f32("emissivity", self.emissivity);
                uniforms.set_f32("roughness", self.roughness);
                uniforms.set_f32("metallic", self.metallic);
                uniforms.set_f32("ao_strength", self.ao);
                uniforms.set_vec2f32("uv_scale", self.scale);
            }),
        };
        pipeline.insert(mat)
    }
}
