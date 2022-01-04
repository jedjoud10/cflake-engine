use crate::basics::*;
use crate::object::{PipelineObjectID, PipelineObject, PipelineTask, ObjectID};
use crate::pipeline::*;
use bitflags::bitflags;

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
    shader: ObjectID<Shader>,
    flags: MaterialFlags,
    uniforms: ShaderUniformsGroup,
    _private: ()
}

impl Buildable for Material {
    fn send(self, pipeline: &Pipeline) -> ObjectID<Self> {
        // Create the ID
        let id = pipeline.materials.get_next_idx_increment();
        ObjectID::new(id)
    }

    fn new(pipeline: &Pipeline) -> Self {
        // Create a new material, with default 
        let mut materail = Self {
            shader: None,
            flags: MaterialFlags::empty(),
            uniforms: ShaderUniformsGroup::new(),
            _private: (),
        };
        // Create some default uniforms
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2f32("uv_scale", veclib::Vector2::ONE);
        group.set_vec3f32("tint", veclib::Vector3::ONE);
        group.set_f32("normals_strength", 1.0);
        group.set_t2d(name, texture, active_texture_id)
        PipelineObjectBuilder::new(default_material);
        material
        .uniforms
        .set_t2d("diffuse_tex", &pipec::texturec(assets::cachec::load("defaults\\textures\\missing_texture.png").unwrap()), 0);
    material
        .uniforms
        .set_t2d("normals_tex", &pipec::texturec(assets::cachec::load("default_normals").unwrap()), 1);
    
    }
}


impl PipelineObjectBuilder<Material> {
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: TextureLoadOptions, pipeline: &SharedPipeline) -> Self {
        // Load the texture by creating it's builder
        let mut builder = assets::assetc::dload::<>(path)

        let texture = assets::assetc::dload::<Texture>(diffuse_path).unwrap()
        let texture = crate::pipec::texturec(pipeline, );
        self.uniforms.set_t2d("diffuse_tex", &texture, 0);
        self
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: TextureLoadOptions) -> Self {
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
