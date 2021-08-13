use crate::engine::core::ecs::component::{Component, ComponentID};
use crate::engine::core::{ecs::*, world::World};
use crate::engine::rendering::model::{Model, ModelDataGPU};
use crate::engine::rendering::renderer::{EntityRenderState, Renderer};
use glam::Vec4Swizzles;

// A simple camera component
pub struct Camera {
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
    pub horizontal_fov: f32,
    pub aspect_ratio: f32,
    pub clip_planes: (f32, f32), // Near, far
    pub window_size: (i32, i32), // Width, height
}

// Impl block for Camera component
impl Camera {
    // Update the projection matrix of this camera
    pub fn update_projection_matrix(&mut self) {
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0
            * ((self.horizontal_fov.to_radians() / 2.0).tan()
                * (self.window_size.1 as f32 / self.window_size.0 as f32))
                .atan();
        self.projection_matrix = glam::Mat4::perspective_rh(
            vertical_fov,
            self.aspect_ratio,
            self.clip_planes.0,
            self.clip_planes.1,
        );
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: glam::Vec3, rotation: glam::Quat) {
        let rotation_matrix = glam::Mat4::from_quat(rotation);
        let forward_vector = rotation_matrix
            .mul_vec4(glam::vec4(0.0, 0.0, -1.0, 1.0))
            .xyz();
        let up_vector = rotation_matrix
            .mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0))
            .xyz();
        self.view_matrix = glam::Mat4::look_at_rh(position, forward_vector + position, up_vector);
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
            horizontal_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            clip_planes: (0.1, 1000.0),
            window_size: World::get_default_window_size(),
        }
    }
}

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    pub render_state: EntityRenderState,
    pub gpu_data: ModelDataGPU,
    pub shader_name: String,
    pub model: Model,
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
        String::from("Skyshpere")
    }
}
