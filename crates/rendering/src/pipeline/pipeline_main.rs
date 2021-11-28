use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use crate::{Pipeline};


pub struct StaticMut<T> {
    opt: Option<T>,
}

impl<T> StaticMut<T> {
    // Set
    pub fn set(&mut self, new: T) {
        self.opt = Some(new);
    }
    // Get mut
    pub fn as_mut(&mut self) -> &mut T {
        self.opt.as_mut().unwrap()
    }
    pub const EMPTY: Self = Self { opt: None };
}

// Static mut RenderPipeline
pub static mut RENDER_PIPELINE: StaticMut<Pipeline> = StaticMut::EMPTY;

pub fn rname(prefix: &str) -> String {
    // Create a randomized name for a texture without a name
    let mut hash = DefaultHasher::new();
    let st = std::time::SystemTime::now();
    st.hash(&mut hash);
    let x = hash.finish();
    format!("{}_{:x}", prefix, x).to_string()    
}

pub mod pipec {
    use std::ffi::c_void;

    use assets::{CachedObject};

    use crate::pipeline::object::*;
    use crate::{Model, Pipeline, RenderTaskReturn, RenderTaskStatus, Renderer, Shader, SubShader, Texture, RENDER_PIPELINE};
    pub use crate::{RenderTask, SharedData};
    // Start the render pipeline by initializing OpenGL on the new render thread
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        unsafe {
            RENDER_PIPELINE.set(Pipeline::default());
            RENDER_PIPELINE.as_mut().init_pipeline(glfw, window);
        }
    }
    pub fn start_world() {
        unsafe {
            RENDER_PIPELINE.as_mut().start_world();
        }
    }
    pub fn frame_main_thread() {
        unsafe {
            RENDER_PIPELINE.as_mut().frame_main_thread();
        }
    }
    // Dispose of the render thread and render pipeline
    pub fn dispose_pipeline() {
        unsafe {
            RENDER_PIPELINE.as_mut().dispose_pipeline();
        }
    }
    // Immediate Task
    pub fn task_immediate(task: RenderTask, name: &str) -> Option<RenderTaskReturn> {
        unsafe { RENDER_PIPELINE.as_mut().task_immediate(task, name.to_string()) }
    }
    // Normal callback task
    pub fn task<F>(task: RenderTask, name: &str, callback: F)
    where
        F: FnMut(RenderTaskStatus) + 'static,
    {
        unsafe { RENDER_PIPELINE.as_mut().task(task, name.to_string(), callback) }
    }
    // Internal task
    pub fn internal_task(task: RenderTask, name: &str) -> Option<RenderTaskReturn> {
        unsafe { RENDER_PIPELINE.as_mut().internal_task_immediate(task, name.to_string()) }
    }
    // Task immmediate, with the inner GPU object
    fn task_immediate_gpuobject(task: RenderTask, name: &str) -> Option<GPUObject> {
        match task_immediate(task, name) {
            Some(x) => match x {
                RenderTaskReturn::GPUObject(x) => Some(x),
                _ => None,
            },
            None => None,
        }
    }
    // Internal immediate task, with the inner GPU object
    fn internal_task_gpuobject(task: RenderTask, name: &str) -> Option<GPUObject> {
        match internal_task(task, name) {
            Some(x) => match x {
                RenderTaskReturn::GPUObject(x) => Some(x),
                _ => None,
            },
            None => None,
        }
    }
    // Actual commands start here
    fn create_texture(texture: Texture) -> TextureGPUObject {
        let name = texture.name.clone();
        match task_immediate_gpuobject(RenderTask::TextureCreate(SharedData::new(texture)), &format!("crt_txtre_{}", name)).unwrap() {
            GPUObject::Texture(x) => x,
            _ => panic!(),
        }
    }
    fn create_subshader(subshader: SubShader) -> SubShaderGPUObject {
        let name = subshader.name.clone();
        match task_immediate_gpuobject(RenderTask::SubShaderCreate(SharedData::new(subshader)), &format!("crt_sbshdr_{}", name)).unwrap() {
            GPUObject::SubShader(x) => x,
            _ => panic!(),
        }
    }
    fn create_shader(shader: Shader) -> ShaderGPUObject {
        let name = shader.name.clone();
        match task_immediate_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader)), &format!("crt_shdr_{}", name)).unwrap() {
            GPUObject::Shader(x) => x,
            _ => panic!(),
        }
    }
    fn create_compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        let name = shader.name.clone();
        match task_immediate_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader)), &format!("crt_cmptshdr_{}", name)).unwrap() {
            GPUObject::ComputeShader(x) => x,
            _ => panic!(),
        }
    }
    fn create_model(model: Model) -> ModelGPUObject {
        let name = model.name.clone();
        match task_immediate_gpuobject(RenderTask::ModelCreate(SharedData::new(model)), &format!("crt_mdl{}", name)).unwrap() {
            GPUObject::Model(x) => x,
            _ => panic!(),
        }
    }
    fn get_gpu_object(name: &str) -> Option<GPUObject> {
        unsafe { RENDER_PIPELINE.as_mut().get_gpu_object(name).cloned() }
    }
    pub fn gpu_object_valid(name: &str) -> bool {
        unsafe { RENDER_PIPELINE.as_mut().gpu_object_valid(name) }
    }

    pub fn get_subshader_object(name: &str) -> SubShaderGPUObject {
        if let GPUObject::SubShader(x) = get_gpu_object(name).unwrap() {
            x
        } else {
            panic!()
        }
    }
    pub fn get_shader_object(name: &str) -> ShaderGPUObject {
        if let GPUObject::Shader(x) = get_gpu_object(name).unwrap() {
            x
        } else {
            panic!()
        }
    }
    pub fn get_compute_shader_object(name: &str) -> ComputeShaderGPUObject {
        if let GPUObject::ComputeShader(x) = get_gpu_object(name).unwrap() {
            x
        } else {
            panic!()
        }
    }
    pub fn get_model_object(name: &str) -> ModelGPUObject {
        if let GPUObject::Model(x) = get_gpu_object(name).unwrap() {
            x
        } else {
            panic!()
        }
    }
    pub fn get_texture_object(name: &str) -> TextureGPUObject {
        if let GPUObject::Texture(x) = get_gpu_object(name).unwrap() {
            x
        } else {
            panic!()
        }
    }
    // Load or create functions
    pub fn subshader(subshader: SubShader) -> SubShaderGPUObject {
        if gpu_object_valid(&subshader.name) {
            get_subshader_object(&subshader.name)
        } else {
            create_subshader(subshader)
        }
    }
    pub fn shader(shader: Shader) -> ShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_shader_object(&shader.name)
        } else {
            create_shader(shader)
        }
    }
    pub fn compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_compute_shader_object(&shader.name)
        } else {
            create_compute_shader(shader)
        }
    }
    pub fn texture(texture: Texture) -> TextureGPUObject {
        if gpu_object_valid(&texture.name) {
            get_texture_object(&texture.name)
        } else {
            create_texture(texture)
        }
    }
    pub fn model(model: Model) -> ModelGPUObject {
        // (TODO: Implement model caching)
        create_model(model)
    }
    // And their internal counterpart, whoever, these don't cache/load. They alway do the conversion
    pub fn isubshader(subshader: SubShader) -> SubShaderGPUObject {
        let name = subshader.name.clone();
        match internal_task_gpuobject(RenderTask::SubShaderCreate(SharedData::new(subshader)), &format!("icrt_sbshdr_{}", name)).unwrap() {
            GPUObject::SubShader(x) => x,
            _ => panic!(),
        }
    }
    pub fn ishader(shader: Shader) -> ShaderGPUObject {
        let name = shader.name.clone();
        match internal_task_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader)), &format!("icrt_shdr_{}", name)).unwrap() {
            GPUObject::Shader(x) => x,
            _ => panic!(),
        }
    }
    pub fn icompute_shader(shader: Shader) -> ComputeShaderGPUObject {
        let name = shader.name.clone();
        match internal_task_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader)), &format!("icrt_cmptshdr_{}", name)).unwrap() {
            GPUObject::ComputeShader(x) => x,
            _ => panic!(),
        }
    }
    pub fn itexture(texture: Texture) -> TextureGPUObject {
        let name = texture.name.clone();
        match internal_task_gpuobject(RenderTask::TextureCreate(SharedData::new(texture)), &format!("icrt_txtre_{}", name)).unwrap() {
            GPUObject::Texture(x) => x,
            _ => panic!(),
        }
    }
    pub fn imodel(model: Model) -> ModelGPUObject {
        let name = model.name.clone();
        match internal_task_gpuobject(RenderTask::ModelCreate(SharedData::new(model)), &format!("icrt_mdl_{}", name)).unwrap() {
            GPUObject::Model(x) => x,
            _ => panic!(),
        }
    }

    // Load or create functions, cached type
    pub fn texturec(texturec: CachedObject<Texture>) -> TextureGPUObject {
        if gpu_object_valid(&texturec.arc.name) {
            get_texture_object(&texturec.arc.name)
        } else {
            let texture = texturec.arc.as_ref().clone();
            create_texture(texture)
        }
    }
    pub fn shaderc(shaderc: CachedObject<Shader>) -> ShaderGPUObject {
        if gpu_object_valid(&shaderc.arc.name) {
            get_shader_object(&shaderc.arc.name)
        } else {
            let shader = shaderc.arc.as_ref().clone();
            create_shader(shader)
        }
    }
    // Load or create functions, cached type, internal
    pub fn itexturec(texturec: CachedObject<Texture>) -> TextureGPUObject {
        if gpu_object_valid(&texturec.arc.name) {
            get_texture_object(&texturec.arc.name)
        } else {
            let texture = texturec.arc.as_ref().clone();
            itexture(texture)
        }
    }
    // Read the data from an array that was filled using a texture
    pub fn convert_native<T>(taskreturn: RenderTaskReturn) -> Vec<T>
    where
        T: Default + Clone + Sized,
    {
        // Convert the bytes into a vector of vectors
        let (bytes, _) = match taskreturn {
            RenderTaskReturn::GPUObject(x) => match x {
                GPUObject::TextureFill(x) => (x.0, x.1),
                _ => panic!()
            },
            _ => panic!()
        };
        // Unsafe
        let t = bytes.chunks_exact(std::mem::size_of::<T>())
            .map(|x| unsafe { 
                std::ptr::read::<T>(x.as_ptr() as *const _)
            }
        );
        let pixels: Vec<T> = t.collect();
        pixels
    }
    pub fn convert_native_veclib<T, U>(taskreturn: RenderTaskReturn) -> Vec<T>
    where
        T: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Convert the bytes into a vector of vectors
        let (bytes, _) = match taskreturn {
            RenderTaskReturn::GPUObject(x) => match x {
                GPUObject::TextureFill(x) => (x.0, x.1),
                _ => panic!()
            },
            _ => panic!()
        };
        // Unsafe
        let t = bytes.chunks_exact(std::mem::size_of::<T>())
            .map(|x| unsafe { 
                std::ptr::read::<T>(x.as_ptr() as *const _)
            }
        );
        let pixels: Vec<T> = t.collect();
        pixels
    }
    // Renderers
    pub fn add_renderer(renderer: Renderer, matrix: veclib::Matrix4x4<f32>) -> usize {
        match task_immediate_gpuobject(RenderTask::RendererAdd(SharedData::new((renderer, matrix))), "").unwrap() {
            GPUObject::Renderer(x) => x,
            _ => panic!(),
        }
    }
    pub fn remove_renderer(index: usize) {
        task(RenderTask::RendererRemove(index), "", |x| {});
    }
}
