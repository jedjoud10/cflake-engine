use std::{ffi::c_void, mem::size_of, ptr::null};

use super::{model::Model, model::ModelDataGPU, shader::Shader, texture::Texture};
use crate::engine::{
    core::{
        cacher::CacheManager,
        ecs::component::{Component, ComponentID},
        world::World,
    },
    resources::ResourceManager,
};
// A component that will be linked to entities that are renderable
pub struct Renderer {
    pub render_state: EntityRenderState,
    pub gpu_data: ModelDataGPU,
    pub shader_name: String,
    pub model: Model,
    // Rendering stuff
    pub texture_cache_ids: Vec<u16>,
    // Default parameters for the shader
    pub uv_scale: glam::Vec2,
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
        }
    }
}

// Main traits implemented
impl Component for Renderer {
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

// Everything related to the creation of a renderer
impl Renderer {
    // Load a model
    pub fn load_model(&mut self, model_path: &str, resource_manager: &mut ResourceManager) {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::from_resource(resource).unwrap();
        self.model = model;
    }
    // Load textures
    pub fn load_textures(
        &mut self,
        texture_paths: Vec<&str>,
        texture_manager: &mut CacheManager<Texture>,
        resource_manager: &mut ResourceManager,
    ) {
        // Load the textures
        for (i, &texture_path) in texture_paths.iter().enumerate() {
            let resource = resource_manager.load_packed_resource(texture_path).unwrap();
            let mut texture = Texture::new().set_mutable(false).enable_mipmaps().set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE).load_texture(texture_path, resource_manager, texture_manager).unwrap();
            self.texture_cache_ids.push(texture_manager.get_object_id(texture_path).unwrap());
        }

        // For the rest of the textures that weren't explicitly given a texture path, load the default ones
        // Diffuse, Normals, Roughness, Metallic, AO
        for i in [(texture_paths.len() - 1)..5] {
            self.texture_cache_ids.push(
                texture_manager
                    .get_object_id("textures\\white.png")
                    .unwrap(),
            );
        }
    }
}

impl Renderer {
    // Updates the model matrix using a position and a rotation
    pub fn update_model_matrix(&mut self, position: glam::Vec3, rotation: glam::Quat, scale: f32) {
        let model_matrix = glam::Mat4::from_quat(rotation)
            * glam::Mat4::from_translation(position)
            * glam::Mat4::from_scale(glam::vec3(scale, scale, scale));
        self.gpu_data.model_matrix = model_matrix;
    }
    // When we update the model and want to refresh it's OpenGL data
    pub fn refresh_model(&mut self) {
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut self.gpu_data.vertex_array_object);
            gl::BindVertexArray(self.gpu_data.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut self.gpu_data.element_buffer_object);
            gl::BindBuffer(
                gl::ELEMENT_ARRAY_BUFFER,
                self.gpu_data.element_buffer_object,
            );
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
            println!(
                "Initialized model with '{}' vertices and '{}' triangles",
                self.model.vertices.len(),
                self.model.triangles.len()
            );
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
pub enum EntityRenderState {
    Visible,
    Invisible,
}

impl Default for EntityRenderState {
    fn default() -> Self {
        Self::Visible
    }
}
