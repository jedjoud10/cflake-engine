use std::{ffi::c_void, mem::size_of, ptr::null};

use super::{model::Model, model::ModelDataGPU, texture::Texture};
use crate::engine::{core::{cacher::CacheManager, defaults::components, ecs::component::{Component, ComponentID, ComponentInternal}}, resources::ResourceManager};
use bitflags::bitflags;

bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}
// A component that will be linked to entities that are renderable
#[derive(Debug)]
pub struct Renderer {
    pub render_state: EntityRenderState,
    pub gpu_data: ModelDataGPU,
    pub shader_name: String,
    pub model: Model,
    // Rendering stuff
    pub texture_cache_ids: Vec<u16>,
    // Default parameters for the shader
    pub uv_scale: glam::Vec2,
    // Flags
    pub flags: RendererFlags,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            render_state: EntityRenderState::Visible,
            gpu_data: ModelDataGPU::default(),
            shader_name: String::default(),
            model: Model::default(),
            texture_cache_ids: Vec::new(),
            uv_scale: glam::Vec2::ONE,
            flags: RendererFlags::DEFAULT,
        }
    }
}

// Main traits implemented
impl ComponentInternal for Renderer {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for Renderer {
    fn get_component_name() -> String {
        String::from("Render")
    }
}
impl Component for Renderer {}

// Everything related to the creation of a renderer
impl Renderer {
    // Create a renderer
    pub fn new() -> Self {
        Self::default()
    }
    // Load a model
    pub fn load_model(mut self, model_path: &str, resource_manager: &mut ResourceManager) -> Self {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::from_resource(resource).unwrap();
        self.model = model;
        return self;
    }
    // Set a model
    pub fn set_model(mut self, model: Model) -> Self {
        self.model = model;
        return self;
    }
    // Set the main shader
    pub fn set_shader(mut self, shader_name: &str) -> Self {
        self.shader_name = shader_name.to_string();
        return self;
    }
    // Enable / disable the wireframe rendering for this entity
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.flags.remove(RendererFlags::WIREFRAME);
        }
        return self;
    }
    // Load textures from their resource paths
    pub fn resource_load_textures(mut self, texture_paths: Vec<&str>, texture_cacher: &mut CacheManager<Texture>, resource_manager: &mut ResourceManager) -> Self {
        // Load the textures
        for (_i, &texture_path) in texture_paths.iter().enumerate() {
            let _resource = resource_manager.load_packed_resource(texture_path).unwrap();
            let _texture = Texture::new()
                .set_mutable(true)
                .enable_mipmaps()
                .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
                .load_texture(texture_path, resource_manager, texture_cacher)
                .unwrap();
            self.texture_cache_ids.push(texture_cacher.get_object_id(texture_path).unwrap());
        }
        // Load the default textures
        return self.load_default_textures(texture_cacher);
    }
    // Load textures from their texture struct
    pub fn load_textures(mut self, texture_ids: Vec<u16>, texture_cacher: &CacheManager<Texture>) -> Self {
        // Set the textures as the renderer's textures
        for (&texture_id) in texture_ids.iter() {
            // Since these are loadable textures, we already know they got cached beforehand
            self.texture_cache_ids.push(texture_id);
        }
        // Load the default textures
        return self.load_default_textures(texture_cacher);
    }
    // Load the default textures
    pub fn load_default_textures(mut self, texture_cacher: &CacheManager<Texture>) -> Self {
        // For the rest of the textures that weren't explicitly given a texture path, load the default ones
        // Diffuse, Normals, Roughness, Metallic, AO
        for _i in (self.texture_cache_ids.len())..5 {
            self.texture_cache_ids.push(texture_cacher.get_object_id("textures\\white.png").unwrap());
        }
        return self;
    }
}

impl Renderer {
    // When we update the model and want to refresh it's OpenGL data
    pub fn refresh_model(&mut self) {
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut self.gpu_data.vertex_array_object);
            gl::BindVertexArray(self.gpu_data.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut self.gpu_data.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.gpu_data.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.model.triangles.len() * size_of::<u32>()) as isize,
                self.model.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut self.gpu_data.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.vertices.len() * size_of::<f32>() * 3) as isize,
                self.model.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut self.gpu_data.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.normals.len() * size_of::<f32>() * 3) as isize,
                self.model.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // And it's brother, the tangent buffer
            gl::GenBuffers(1, &mut self.gpu_data.tangent_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.tangent_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.tangents.len() * size_of::<f32>() * 4) as isize,
                self.model.tangents.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Finally, the texture coordinates buffer
            gl::GenBuffers(1, &mut self.gpu_data.uv_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.uv_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.uvs.len() * size_of::<f32>() * 2) as isize,
                self.model.uvs.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Tangent attribute
            gl::EnableVertexAttribArray(2);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.tangent_buf);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());

            // UV attribute
            gl::EnableVertexAttribArray(3);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.uv_buf);
            gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());

            self.gpu_data.initialized = true;
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    // Dispose of our model
    pub fn dispose_model(&mut self) {
        unsafe {
            // Delete the vertex array
            gl::DeleteBuffers(1, &mut self.gpu_data.vertex_buf);
        }
    }
}

// The current render state of the entity
#[derive(Debug)]
pub enum EntityRenderState {
    Visible,
    Invisible,
}

// If the entity is culled or not
#[derive(Debug)]
pub enum EntityCullingState {
    Culled,
    Unculled,
}

impl Default for EntityRenderState {
    fn default() -> Self {
        Self::Visible
    }
}
