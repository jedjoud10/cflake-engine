use crate::engine::core::defaults::components::transforms;
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentManager};
use crate::engine::core::ecs::entity::Entity;
use crate::engine::math::bounds;
use crate::engine::rendering::model::Model;
use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::window::Window;
use glam::Vec4Swizzles;

// A simple camera component
pub struct Camera {
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
    pub frustum_culling_matrix: glam::Mat4,
    pub horizontal_fov: f32,
    pub aspect_ratio: f32,
    pub clip_planes: (f32, f32), // Near, far
}

// Impl block for Camera component
impl Camera {
    // Update the projection matrix of this camera
    pub fn update_projection_matrix(&mut self, window: &Window) {
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0 * ((self.horizontal_fov.to_radians() / 2.0).tan() * (window.size.1 as f32 / window.size.0 as f32)).atan();
        self.projection_matrix = glam::Mat4::perspective_rh(vertical_fov, self.aspect_ratio, self.clip_planes.0, self.clip_planes.1);
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: glam::Vec3, rotation: glam::Quat) {
        let rotation_matrix = glam::Mat4::from_quat(rotation);
        let forward_vector = rotation_matrix.mul_vec4(glam::vec4(0.0, 0.0, -1.0, 1.0)).xyz();
        let up_vector = rotation_matrix.mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0)).xyz();
        self.view_matrix = glam::Mat4::look_at_rh(position, forward_vector + position, up_vector);
    }
    // Update the frustum-culling matrix
    pub fn update_frustum_culling_matrix(&mut self) {
        // Too ez m8
        self.frustum_culling_matrix = self.projection_matrix * self.view_matrix;
    }
}

impl Component for Camera {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Camera {
    fn get_component_name() -> String {
        String::from("Camera Component")
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::IDENTITY,
            frustum_culling_matrix: glam::Mat4::IDENTITY,
            horizontal_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            clip_planes: (3.0, 10000.0),
        }
    }
}

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    pub sky_gradient_texture_id: u16,
}

// Main traits implemented
impl Component for Sky {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for Sky {
    fn get_component_name() -> String {
        String::from("Sky")
    }
}

// An AABB components
#[derive(Default)]
pub struct AABB {
    pub aabb: bounds::AABB,
    pub generation_type: AABBGenerationType
}

// How we are going to generate the AABB
pub enum AABBGenerationType {
    RenderEntity,
    Manual
}

// Automatically try to load the AABB from the components of a render entity (Position, Scale, Render)
impl Default for AABBGenerationType { 
    fn default() -> Self { Self::RenderEntity }
}

// AABB component functions
impl AABB {
    // Generate the AABB from a renderer entity
    pub fn from_components(entity: &Entity, component_manager: &ComponentManager) -> Self {
        let model_ref = &entity.get_component::<Renderer>(component_manager).unwrap().model;
        let position = entity.get_component::<transforms::Position>(component_manager).unwrap().position;
        let scale = entity.get_component::<transforms::Scale>(component_manager).unwrap().scale;
        let mut aabb = bounds::AABB::from_model(model_ref);
        aabb.offset(position);        
        aabb.scale(scale);
        println!("AABB generated from model, min: {} max: {}", aabb.min, aabb.max);
        return Self { 
            aabb,
            ..Self::default() 
        };
    }
}

// Main traits implemented
impl Component for AABB {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for AABB {
    fn get_component_name() -> String {
        String::from("AABB")
    }
}