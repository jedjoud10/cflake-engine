use crate::Uniform;

use super::{model::Model, model::ModelDataGPU, Material};
use bitflags::bitflags;
use ecs::{Component, ComponentID, ComponentInternal};
use resources::{LoadableResource, ResourceManager};
use std::{ffi::c_void, mem::size_of, ptr::null};

bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}
// A component that will be linked to entities that are renderable
pub struct Renderer {
    pub render_state: EntityRenderState,
    pub gpu_data: ModelDataGPU,
    pub model: Model,
    // This renderer can only have one material for now (TODO: Make a multi material system)
    pub material: Option<Material>,
    // Flags
    pub flags: RendererFlags,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            render_state: EntityRenderState::Visible,
            gpu_data: ModelDataGPU::default(),
            model: Model::default(),
            material: None,
            flags: RendererFlags::DEFAULT,
        }
    }
}

// Main traits implemented
ecs::impl_component!(Renderer);

// Everything related to the creation of a renderer
impl Renderer {
    // Create a renderer
    pub fn new() -> Self {
        let new_self = Self::default();
        // Create a default material, just contains the shader arguments only though, no textures
        let material = Material::default();
        // Set the default uniforms
        let material = material.val("uv_scale", Uniform::Vec2F32(veclib::Vector2::ONE));
        let material = material.val("tint", Uniform::Vec3F32(veclib::Vector3::ONE));
        let material = material.val("normals_strength", Uniform::F32(1.0));
        return new_self.set_material(material);
    }
    // Load a model
    pub fn load_model(mut self, model_path: &str, resource_manager: &mut ResourceManager) -> Self {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::new().from_resource(resource).unwrap();
        self.model = model;
        return self;
    }
    // Set a model
    pub fn set_model(mut self, model: Model) -> Self {
        self.model = model;
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
    // With a specific material
    pub fn set_material(mut self, material: Material) -> Self {
        self.material = Some(material);
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

            // The texture coordinates buffer
            gl::GenBuffers(1, &mut self.gpu_data.uv_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.uv_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.uvs.len() * size_of::<f32>() * 2) as isize,
                self.model.uvs.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            // Finally, the vertex colors buffer
            gl::GenBuffers(1, &mut self.gpu_data.color_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.color_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.model.colors.len() * size_of::<f32>() * 3) as isize,
                self.model.colors.as_ptr() as *const c_void,
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

            // Vertex color attribute
            gl::EnableVertexAttribArray(4);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.color_buf);
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());

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
