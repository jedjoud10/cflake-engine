use std::{collections::HashMap, iter::FromIterator};
use crate::{GPUObject, Pipeline};

// Static mut RenderPipeline
pub static mut render_pipeline: Option<Pipeline> = None;

pub mod pipec {
    use assets::{AssetManager, CachedObject};

    use crate::pipeline::object::*;
    use crate::{Model, Pipeline, RenderTask, Shader, SharedData, SubShader, Texture, render_pipeline};
    // Start the render pipeline by initializing OpenGL on the new render thread
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        unsafe {
            render_pipeline = Some(Pipeline::default());
            render_pipeline.unwrap().init_pipeline(glfw, window);
        }
    }
    pub fn start_world(asset_manager: &mut AssetManager) {
        unsafe {
            render_pipeline.unwrap().start_world(asset_manager);
        }
    }
    // Task
    pub fn task_immediate(task: RenderTask) -> GPUObject {
        unsafe {
            render_pipeline.unwrap().task_immediate(task)
        }
    }

    // Actual commands start here
    fn create_texture(texture: Texture) -> TextureGPUObject {
        unsafe {
            match render_pipeline.unwrap().task_immediate(RenderTask::TextureCreate(SharedData::new(texture))) {
                GPUObject::Texture(x) => x,
                _ => panic!(),
            }
        }
    }
    fn create_subshader(subshader: SubShader) -> SubShaderGPUObject {
        unsafe {
            match render_pipeline.unwrap().task_immediate(RenderTask::SubShaderCreate(SharedData::new(subshader))) {
                GPUObject::SubShader(x) => x,
                _ => panic!(),
            }
        }
    }
    fn create_shader(shader: Shader) -> ShaderGPUObject {
        unsafe {
            match render_pipeline.unwrap().task_immediate(RenderTask::ShaderCreate(SharedData::new(shader))) {
                GPUObject::Shader(x) => x,
                _ => panic!(),
            }
        }
    }
    fn create_compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        unsafe {
            match render_pipeline.unwrap().task_immediate(RenderTask::ShaderCreate(SharedData::new(shader))) {
                GPUObject::ComputeShader(x) => x,
                _ => panic!(),
            }
        }
    }
    fn create_model(model: Model) -> ModelGPUObject {
        unsafe {
            match render_pipeline.unwrap().task_immediate(RenderTask::ModelCreate(SharedData::new(model))) {
                GPUObject::Model(x) => x,
                _ => panic!(),
            }
        }
    }
    fn get_gpu_object(name: &str) -> GPUObject {
        unsafe {
            render_pipeline.unwrap().get_gpu_object(name).clone()
        }
    }
    fn gpu_object_valid(name: &str) -> bool {
        unsafe {
            render_pipeline.unwrap().gpu_object_valid(name)
        }
    }
    pub fn get_subshader_object(name: &str) -> SubShaderGPUObject {
        if let GPUObject::SubShader(x) = get_gpu_object(name) { x }  else { panic!() }
    }
    pub fn get_shader_object(name: &str) -> ShaderGPUObject {
        if let GPUObject::Shader(x) = get_gpu_object(name) { x }  else { panic!() } 
    }
    pub fn get_compute_shader_object(name: &str) -> ComputeShaderGPUObject {
        if let GPUObject::ComputeShader(x) = get_gpu_object(name) { x }  else { panic!() } 
    }
    pub fn get_model_object(name: &str) -> ModelGPUObject {
        if let GPUObject::Model(x) = get_gpu_object(name) { x }  else { panic!() }
    }
    pub fn get_texture_object(name: &str) -> TextureGPUObject {
        if let GPUObject::Texture(x) = get_gpu_object(name) { x }  else { panic!() }
    }
    // Load or create functions
    pub fn subshader(subshader: SubShader) -> SubShaderGPUObject {
        if gpu_object_valid(&subshader.name) { get_subshader_object(&subshader.name) }
        else { create_subshader(subshader) }
    }
    pub fn shader(shader: Shader) -> ShaderGPUObject {
        if gpu_object_valid(&shader.name) { get_shader_object(&shader.name) }
        else { create_shader(shader) }
    }
    pub fn compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        if gpu_object_valid(&shader.name) { get_compute_shader_object(&shader.name) }
        else { create_compute_shader(shader) }
    }
    pub fn texture(texture: Texture) -> TextureGPUObject {
        if gpu_object_valid(&texture.name) { get_texture_object(&texture.name) }
        else { create_texture(texture.clone()) }
    }
    pub fn model(model: Model) -> ModelGPUObject {
        // (TODO: Implement model caching)
        create_model(model)
    }
    // Load or create functions, cached type
    pub fn texturec(texturec: CachedObject<Texture>) -> TextureGPUObject {
        if gpu_object_valid(&texturec.rc.name) { get_texture_object(&texturec.rc.name) }
        else { 
            let texture = texturec.rc.as_ref().clone();
            create_texture(texture)
        }
    }
    pub fn shaderc(shaderc: CachedObject<Shader>) -> ShaderGPUObject {
        if gpu_object_valid(&shaderc.rc.name) { get_shader_object(&shaderc.rc.name) }
        else { 
            let shader = shaderc.rc.as_ref().clone();
            create_shader(shader)
        }
    }
}
