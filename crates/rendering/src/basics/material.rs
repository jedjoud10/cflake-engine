use crate::basics::*;
use crate::object::{PipelineObjectID, PipelineObject, PipelineTask};
use crate::pipeline::*;

use bitflags::bitflags;
use others::Context;

bitflags! {
    pub struct MaterialFlags: u8 {
        const DOUBLE_SIDED = 0b00000001;
    }
}
impl Default for MaterialFlags {
    fn default() -> Self {
        Self::empty()
    }
}

// A material that can have multiple parameters and such
pub struct Material {
    // Rendering stuff
    pub name: String,
    pub shader: Option<PipelineObjectID<Shader>>,
    pub flags: MaterialFlags,
    uniforms: ShaderUniformsGroup,
    _private: ()
}

impl PipelineObject for Material {
    // Create a new builder for this material
    fn builder() -> PipelineObjectBuilder<Self> {
        let mut default_material = Self {
            name: crate::utils::rname("material"),
            shader: None,
            flags: MaterialFlags::empty(),
            uniforms: ShaderUniformsGroup::new(),
            _private: (),
        };
        let mut group = ShaderUniformsGroup::new();
        default_material.uniforms.set_vec2f32("uv_scale", veclib::Vector2::ONE);
        default_material.uniforms.set_vec3f32("tint", veclib::Vector3::ONE);
        default_material.uniforms.set_f32("normals_strength", 1.0);
        PipelineObjectBuilder::new(default_material);
    }    
}

impl BuilderConvert for PipelineObjectBuilder<Texture> {
    fn convert(self) -> PipelineTask {
        PipelineTask::CreateMaterial(self)
    }
}


impl PipelineObjectBuilder<Texture> {
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: Option<TextureLoadOptions>, pipeline: Context<SharedPipeline>) -> Self {
        // Load the texture
        let texture = pipec::texturec(
            assets::assetc::load(path, obj)
                diffuse_path,
                Texture::default().enable_mipmaps().set_format(TextureFormat::RGBA8R).apply_texture_load_options(opt),
            )
            .unwrap(),
        );
        self.uniforms.set_t2d("diffuse_tex", &texture, 0);
        self
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: Option<TextureLoadOptions>) -> Self {
        // Load the texture
        let texture = pipec::texturec(
            assets::cachec::acache_l(
                normal_path,
                Texture::default().enable_mipmaps().set_format(TextureFormat::RGBA8R).apply_texture_load_options(opt),
            )
            .unwrap(),
        );
        self.uniforms.set_t2d("normals_tex", &texture, 1);
        self
    }
    // Set the main shader
    pub fn set_shader(mut self, shader: GPUObjectID) -> Self {
        self.shader = Some(shader);
        self
    }
    // Toggle the double sided flag for this material
    pub fn set_double_sided(mut self, double_sided: bool) -> Self {
        match double_sided {
            true => self.flags.insert(MaterialFlags::DOUBLE_SIDED),
            false => self.flags.remove(MaterialFlags::DOUBLE_SIDED),
        }
        self
    }
    // Toggle the visibility of this material
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

// Create a new material with a name
pub fn new(material_name: &str) -> Self {
    let mut material = Self::default();
    material.material_name = material_name.to_string();
    material
        .uniforms
        .set_t2d("diffuse_tex", &pipec::texturec(assets::cachec::load("defaults\\textures\\missing_texture.png").unwrap()), 0);
    material
        .uniforms
        .set_t2d("normals_tex", &pipec::texturec(assets::cachec::load("default_normals").unwrap()), 1);
    
    material
}
