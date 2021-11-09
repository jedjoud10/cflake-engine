use assets::AssetManager;
use math;
use rendering::{Model, Renderer, Shader, SubShader};
use std::{ffi::c_void, mem::size_of, ptr::null};

// Constants
pub const MAX_LINE_COUNT: i32 = 8192;
pub const MAX_DEBUG_PRIMITIVES: usize = 512;
pub const MAX_PERMAMENT_DEBUG_PRIMITIVES: usize = 512;
pub const DRAW_DEBUG: bool = true;
// Debug renderer functionality
#[derive(Default)]
pub struct DebugRenderer {
    pub primitives: Vec<DebugPrimitive>,
    // The model renderers 
    pub renderer: Vec<Renderer>,
    pub shader: Shader,
}

impl DebugRenderer {
    // Generate the vao and load the shader
    pub fn setup_debug_renderer(&mut self, asset_manager: &mut AssetManager) {
        // Set the shader name
        self.shader = Shader::new()
            .load_shader(
                vec!["defaults\\shaders\\others\\debug.vrsh.glsl", "defaults\\shaders\\others\\debug.frsh.glsl"],
                asset_manager,
            )
            .unwrap();
    }
    // Draw the debug renderers
    pub fn draw_debug(&mut self, vp_matrix: &veclib::Matrix4x4<f32>) {
        if !DRAW_DEBUG {
            return;
        }
        
        // Set the shader
        let shader = &mut self.shader;
        // Since we don't have a model matrix you can set it directly
        shader.use_shader();
        shader.set_mat44("vp_matrix", &vp_matrix);
    }
    // Add a debug primitive to the queue and then render it
    pub fn debug(&mut self, debug_primitive: DebugPrimitive) {
        if !DRAW_DEBUG {
            return;
        }
        self.primitives.push(debug_primitive);
    }    
}

// A simple debug primitives
pub struct DebugPrimitive { 
    shape: math::shapes::Shape,
    tint: veclib::Vector3<f32>,
    permament: bool
}

impl DebugPrimitive {
    // Create an empty debug primitive
    pub fn new() -> Self {
        Self {
            shape: math::shapes::Shape::new_cube(veclib::Vector3::ZERO, veclib::Vector3::ONE * 0.5),
            tint: veclib::Vector3::ONE,
            permament: true,
        }
    }
    // Set the tint of this debug primitive
    pub fn set_tint(mut self, tint: veclib::Vector3<f32>) -> Self {
        self.tint = tint;
        self
    }
    // Set the shape of this debug primitive
    pub fn set_shape(mut self, shape: math::shapes::Shape) -> Self {
        self.shape = shape;
        self
    }
    // Set the lifetime of this debug primitive
    pub fn set_lifetime(mut self, permament: bool) -> Self {
        self.permament = permament;
        self
    }
}