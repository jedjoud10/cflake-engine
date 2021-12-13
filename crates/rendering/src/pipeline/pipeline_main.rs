use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}, sync::{RwLock, mpsc::Sender, Mutex, atomic::AtomicUsize}, cell::{RefCell, Cell},
};
use crate::{Pipeline, GlobalInterface, RenderCommand};
use lazy_static::lazy_static;

lazy_static! {
    // The number of commands that have been sent
    pub static ref COMMAND_COUNT: AtomicUsize = AtomicUsize::new(0);
}

thread_local! {
    // The render task sender!
    pub static RENDER_COMMAND_SENDER: RefCell<Option<Sender<RenderCommand>>> = RefCell::new(None);
    pub static IS_RENDER_THREAD: Cell<bool> = Cell::new(false);
}

pub fn rname(prefix: &str) -> String {
    // Create a randomized name for a texture without a name
    let mut hash = DefaultHasher::new();
    let st = std::time::SystemTime::now();
    st.hash(&mut hash);
    let x = hash.finish();
    format!("{}_{:x}", prefix, x).to_string()
}

pub mod pipec {
    use std::borrow::{BorrowMut, Borrow};
    use std::ffi::c_void;
    use std::sync::{Arc, Mutex, Condvar};
    use std::sync::atomic::Ordering;
    use assets::CachedObject;

    use crate::pipeline::object::*;
    use crate::{Model, Pipeline, RenderTaskReturn, Renderer, Shader, SubShader, Texture, Material, RenderCommand, COMMAND_COUNT, SubShaderType, RENDER_COMMAND_SENDER};
    pub use crate::{RenderTask, SharedData};
    pub use crate::pipeline::global_interface::*;
    // Start the render pipeline by initializing OpenGL on the new render thread (Ran on the main thread)
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        crate::pipeline::init_pipeline(glfw, window);
    }
    // Ran on the main thread
    pub fn start_world() {
        // Default shader
        let ds = shader(
            Shader::default()
                .load_shader(vec![
                    "defaults\\shaders\\rendering\\passthrough.vrsh.glsl",
                    "defaults\\shaders\\rendering\\screen.frsh.glsl",
                ])
                .unwrap(),
        );
        // Default material
        let _dm = Material::new("Default material").set_shader(ds);
        println!("Loaded default shader and default material!");
    }
    // Just setup the sender of commands thread-locally
    pub fn initialize_threadlocal_render_comms() {
        let pipeline = crate::pipeline::pipeline();
        let tx = pipeline.as_ref().unwrap().tx_template.clone();
        RENDER_COMMAND_SENDER.with(|sender_refcell | {
            let mut sender_ = sender_refcell.borrow_mut();
            let sender = &mut *sender_;
            *sender = Some(tx);
        });
        println!("Initialized the thread local RenderCommand sender!");
    }
    // Dispose of the render thread and render pipeline
    pub fn dispose_pipeline() {
        //self.task_immediate(RenderTask::DestroyRenderThread(), "dispose_pipeline".to_string());
    }
    // Generate a command name
    pub fn generate_command_name() -> String {
        format!("c_{}", COMMAND_COUNT.fetch_add(1, Ordering::Relaxed))
    }
    // Normal callback task
    pub fn task<F>(task: RenderTask, name: &str, callback: F)
    where
        F: FnMut(GPUObject) + 'static + Sync + Send,
    {
        // Create the render command
        let command = RenderCommand {
            name: if name.is_empty() { panic!() } else { name.to_string() },
            input_task: task,
        };
        crate::RENDER_COMMAND_SENDER.with(|x| {
            let sender_ = x.borrow();
            let sender_ = sender_.as_ref();
            let sender = sender_.unwrap();
            // Send the command to the thread
            sender.send(command).unwrap();
        });
    }
    // Internal task
    pub fn internal_task(task: RenderTask, name: &str) -> Option<RenderTaskReturn> {
        // We must talk to the global interface directly
        todo!()
    }
    // We must ask the Interface if we have these objects in cache
    fn get_gpu_object(name: &str) -> Option<GPUObject> { crate::pipeline::global_interface::get_gpu_object(name)  }
    pub fn gpu_object_valid(name: &str) -> bool { crate::pipeline::global_interface::gpu_object_valid(name) }
    // Retrieve these objects from cache
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
            let name = format!("crt_sbshdr_{}", subshader.name.clone());
            task(RenderTask::SubShaderCreate(SharedData::new(subshader)), &name, |_| {});
            if let GPUObject::SubShader(x) = crate::wait_fetch_threadlocal_callbacks_specific(&name) { x } else { panic!() }        
        }
    }
    pub fn shader(shader: Shader) -> ShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_shader_object(&shader.name)
        } else {
            let name = format!("crt_shdr_{}", shader.name.clone());
            task(RenderTask::ShaderCreate(SharedData::new(shader)), &name, |_| {});
            if let GPUObject::Shader(x) = crate::wait_fetch_threadlocal_callbacks_specific(&name) { x } else { panic!() }      
        }
    }
    pub fn compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_compute_shader_object(&shader.name)
        } else {
            let name = format!("crt_cmptshdr_{}", shader.name.clone());
            task(RenderTask::ShaderCreate(SharedData::new(shader)), &name, |_| {});
            if let GPUObject::ComputeShader(x) = crate::wait_fetch_threadlocal_callbacks_specific(&name) { x } else { panic!() }
        }
    }
    pub fn texture(texture: Texture) -> TextureGPUObject {
        if gpu_object_valid(&texture.name) {
            get_texture_object(&texture.name)
        } else {
            let name = format!("crt_txtre_{}", texture.name.clone());
            task(RenderTask::TextureCreate(SharedData::new(texture)), &name, |_| {});
            if let GPUObject::Texture(x) = crate::wait_fetch_threadlocal_callbacks_specific(&name) { x } else { panic!() }
        }
    }
    pub fn model(model: Model) -> ModelGPUObject {
        // (TODO: Implement model caching)
        let name = format!("crt_mdl{}", model.name.clone());
        task(RenderTask::ModelCreate(SharedData::new(model)), &name, |_| {});
        if let GPUObject::Model(x) = crate::wait_fetch_threadlocal_callbacks_specific(&name) { x } else { panic!() }
    }

    // Load or create functions, cached type
    pub fn texturec(texturec: CachedObject<Texture>) -> TextureGPUObject {
        if gpu_object_valid(&texturec.arc.name) {
            get_texture_object(&texturec.arc.name)
        } else {
            let t = texturec.arc.as_ref().clone();
            texture(t)
        }
    }
    pub fn shaderc(shaderc: CachedObject<Shader>) -> ShaderGPUObject {
        if gpu_object_valid(&shaderc.arc.name) {
            get_shader_object(&shaderc.arc.name)
        } else {
            let s = shaderc.arc.as_ref().clone();
            shader(s)
        }
    }
    // Read the data from an array that was filled using a texture
    pub fn convert_native<T>(gpuobject: GPUObject) -> Vec<T>
    where
        T: Default + Clone + Sized,
    {
        // Convert the bytes into a vector of vectors
        let (bytes, _) = match gpuobject {
            GPUObject::TextureFill(x) => (x.0, x.1),
            _ => panic!(),
        };
        // Unsafe
        let t = bytes.chunks_exact(std::mem::size_of::<T>()).map(|x| unsafe { std::ptr::read::<T>(x.as_ptr() as *const _) });
        let pixels: Vec<T> = t.collect();
        pixels
    }
    pub fn convert_native_veclib<T, U>(gpuobject: GPUObject) -> Vec<T>
    where
        T: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Convert the bytes into a vector of vectors
        let (bytes, _) = match gpuobject {
            GPUObject::TextureFill(x) => (x.0, x.1),
            _ => panic!(),
        };
        // Unsafe
        let t = bytes.chunks_exact(std::mem::size_of::<T>()).map(|x| unsafe { std::ptr::read::<T>(x.as_ptr() as *const _) });
        let pixels: Vec<T> = t.collect();
        pixels
    }
}
