use assets::{Asset, AssetManager};
use math;
use others::Instance;
use rendering::{Material, Model, Renderer, Shader, SubShader, Uniform};
use std::{ffi::c_void, mem::size_of, ptr::null, rc::Rc};

pub const DRAW_DEBUG: bool = true;
// Debug renderer functionality
#[derive(Default)]
pub struct DebugRenderer {
    pub primitives: Vec<DebugPrimitive>,
    // The template models
    pub template_models: Vec<Model>,
    // The model renderers
    pub renderers: Vec<(Renderer, veclib::Matrix4x4<f32>)>,
    pub template_material: Material,
}

impl DebugRenderer {
    // Generate the vao and load the shader
    pub fn setup_debug_renderer(&mut self, asset_manager: &mut AssetManager) {
        // Set the shader name
        let shader = Shader::new()
            .load_shader(
                vec!["defaults\\shaders\\others\\debug.vrsh.glsl", "defaults\\shaders\\others\\debug.frsh.glsl"],
                asset_manager,
            )
            .unwrap();
        // Create the material
        self.template_material = Material::new("Debug material", asset_manager).set_shader(Rc::new(shader));
        // Load the template models
        self.template_models
            .push(Model::default().load_asset("defaults\\models\\cube.mdl3d", &asset_manager.asset_cacher).unwrap());
        self.template_models
            .push(Model::default().load_asset("defaults\\models\\sphere.mdl3d", &asset_manager.asset_cacher).unwrap());
    }
    // Add a debug primitive to the queue and then render it
    pub fn debug(&mut self, debug_primitive: DebugPrimitive) {
        if !DRAW_DEBUG {
            return;
        }
        // Add the Renderer first
        let template_model = match debug_primitive.shape.internal_shape {
            math::shapes::ShapeType::Cube(_) => self.template_models.get(0),
            math::shapes::ShapeType::Sphere(_) => self.template_models.get(1),
            math::shapes::ShapeType::AxisPlane(_, _) => todo!(),
        }
        .unwrap()
        .clone();
        let mut renderer = Renderer::new()
            .set_model(template_model)
            .set_wireframe(false)
            .set_material(self.template_material.clone().set_uniform("tint", Uniform::Vec3F32(debug_primitive.tint)));
        renderer.refresh_model();
        // Calculate the model matrix from the position of the primitive and it's size
        let pos_matrix = veclib::Matrix4x4::from_translation(debug_primitive.shape.center);
        let model_matrix = match debug_primitive.shape.internal_shape {
            math::shapes::ShapeType::Cube(x) => pos_matrix * veclib::Matrix4x4::from_scale(x + 1.0),
            math::shapes::ShapeType::Sphere(x) => pos_matrix * veclib::Matrix4x4::from_scale(veclib::Vector3::ONE * x + 1.0),
            math::shapes::ShapeType::AxisPlane(_, _) => todo!(),
        };
        self.primitives.push(debug_primitive);
        self.renderers.push((renderer, model_matrix));
    }
}

// A simple debug primitives
pub struct DebugPrimitive {
    shape: math::shapes::Shape,
    tint: veclib::Vector3<f32>,
    permament: bool,
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
