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

pub mod pipec {
    use assets::{CachedObject};

    use crate::pipeline::object::*;
    use crate::{Model, Pipeline, RenderTask, RenderTaskReturn, RenderTaskStatus, Renderer, Shader, SharedData, SubShader, Texture, RENDER_PIPELINE};
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
    pub fn task_immediate(task: RenderTask) -> Option<RenderTaskReturn> {
        unsafe { RENDER_PIPELINE.as_mut().task_immediate(task) }
    }
    // Normal callback task
    pub fn task<F>(task: RenderTask, callback: F)
    where
        F: FnMut(RenderTaskStatus) + 'static,
    {
        unsafe { RENDER_PIPELINE.as_mut().task(task, callback) }
    }
    // Internal task
    pub fn internal_task(task: RenderTask) -> Option<RenderTaskReturn> {
        unsafe { RENDER_PIPELINE.as_mut().internal_task_immediate(task) }
    }
    // Task immmediate, with the inner GPU object
    fn task_immediate_gpuobject(task: RenderTask) -> Option<GPUObject> {
        match task_immediate(task) {
            Some(x) => match x {
                RenderTaskReturn::GPUObject(x) => Some(x),
                _ => None,
            },
            None => None,
        }
    }
    // Internal immediate task, with the inner GPU object
    fn internal_task_gpuobject(task: RenderTask) -> Option<GPUObject> {
        match internal_task(task) {
            Some(x) => match x {
                RenderTaskReturn::GPUObject(x) => Some(x),
                _ => None,
            },
            None => None,
        }
    }
    // Actual commands start here
    fn create_texture(texture: Texture) -> TextureGPUObject {
        match task_immediate_gpuobject(RenderTask::TextureCreate(SharedData::new(texture))).unwrap() {
            GPUObject::Texture(x) => x,
            _ => panic!(),
        }
    }
    fn create_subshader(subshader: SubShader) -> SubShaderGPUObject {
        match task_immediate_gpuobject(RenderTask::SubShaderCreate(SharedData::new(subshader))).unwrap() {
            GPUObject::SubShader(x) => x,
            _ => panic!(),
        }
    }
    fn create_shader(shader: Shader) -> ShaderGPUObject {
        match task_immediate_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader))).unwrap() {
            GPUObject::Shader(x) => x,
            _ => panic!(),
        }
    }
    fn create_compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        match task_immediate_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader))).unwrap() {
            GPUObject::ComputeShader(x) => x,
            _ => panic!(),
        }
    }
    fn create_model(model: Model) -> ModelGPUObject {
        match task_immediate_gpuobject(RenderTask::ModelCreate(SharedData::new(model))).unwrap() {
            GPUObject::Model(x) => x,
            _ => panic!(),
        }
    }
    fn get_gpu_object(name: &str) -> GPUObject {
        unsafe { RENDER_PIPELINE.as_mut().get_gpu_object(name).clone() }
    }
    fn gpu_object_valid(name: &str) -> bool {
        unsafe { RENDER_PIPELINE.as_mut().gpu_object_valid(name) }
    }

    pub fn get_subshader_object(name: &str) -> SubShaderGPUObject {
        if let GPUObject::SubShader(x) = get_gpu_object(name) {
            x
        } else {
            panic!()
        }
    }
    pub fn get_shader_object(name: &str) -> ShaderGPUObject {
        if let GPUObject::Shader(x) = get_gpu_object(name) {
            x
        } else {
            panic!()
        }
    }
    pub fn get_compute_shader_object(name: &str) -> ComputeShaderGPUObject {
        if let GPUObject::ComputeShader(x) = get_gpu_object(name) {
            x
        } else {
            panic!()
        }
    }
    pub fn get_model_object(name: &str) -> ModelGPUObject {
        if let GPUObject::Model(x) = get_gpu_object(name) {
            x
        } else {
            panic!()
        }
    }
    pub fn get_texture_object(name: &str) -> TextureGPUObject {
        if let GPUObject::Texture(x) = get_gpu_object(name) {
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
        match internal_task_gpuobject(RenderTask::SubShaderCreate(SharedData::new(subshader))).unwrap() {
            GPUObject::SubShader(x) => x,
            _ => panic!(),
        }
    }
    pub fn ishader(shader: Shader) -> ShaderGPUObject {
        match internal_task_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader))).unwrap() {
            GPUObject::Shader(x) => x,
            _ => panic!(),
        }
    }
    pub fn icompute_shader(shader: Shader) -> ComputeShaderGPUObject {
        match internal_task_gpuobject(RenderTask::ShaderCreate(SharedData::new(shader))).unwrap() {
            GPUObject::ComputeShader(x) => x,
            _ => panic!(),
        }
    }
    pub fn itexture(texture: Texture) -> TextureGPUObject {
        match internal_task_gpuobject(RenderTask::TextureCreate(SharedData::new(texture))).unwrap() {
            GPUObject::Texture(x) => x,
            _ => panic!(),
        }
    }
    pub fn imodel(model: Model) -> ModelGPUObject {
        match internal_task_gpuobject(RenderTask::ModelCreate(SharedData::new(model))).unwrap() {
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
    pub fn convert_native<T>(_taskreturn: RenderTaskReturn) -> Vec<T>
    where
        T: Default + Clone + Sized,
    {
        let _bytecount = std::mem::size_of::<T>();
        todo!();
    }
    pub fn convert_native_veclib<T, U>(_taskreturn: RenderTaskReturn) -> Vec<T>
    where
        T: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        let _bytecount = std::mem::size_of::<T>();
        todo!();
    }
    // Renderers
    pub fn add_renderer(renderer: Renderer, matrix: veclib::Matrix4x4<f32>) -> usize {
        match task_immediate_gpuobject(RenderTask::RendererAdd(SharedData::new((renderer, matrix)))).unwrap() {
            GPUObject::Renderer(x) => x,
            _ => panic!(),
        }
    }
    pub fn remove_renderer(index: usize) {
        task_immediate_gpuobject(RenderTask::RendererRemove(index)).unwrap();
    }
}
