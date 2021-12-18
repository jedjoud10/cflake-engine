use lazy_static::lazy_static;
use std::{
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{atomic::AtomicUsize, mpsc::Sender},
};

lazy_static! {
    // The number of commands that have been sent
    pub static ref COMMAND_COUNT: AtomicUsize = AtomicUsize::new(0);
}

thread_local! {
    // The render task sender!
    pub static RENDER_COMMAND_SENDER: RefCell<Option<Sender<crate::PipelineSendData>>> = RefCell::new(None);
    pub static IS_RENDER_THREAD: Cell<bool> = Cell::new(false);
}

// Smol wrapper
pub fn is_render_thread() -> bool {
    IS_RENDER_THREAD.with(|x| x.get())
}

// Generate a random name using the current system time and a prefix
pub fn rname(prefix: &str) -> String {
    // Create a randomized name for a texture without a name
    let mut hash = DefaultHasher::new();
    let st = std::time::SystemTime::now();
    st.hash(&mut hash);
    let x = hash.finish();
    format!("{}_{:x}", prefix, x)
}

pub mod pipec {
    use assets::CachedObject;
    use std::borrow::{Borrow, BorrowMut};

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use crate::pipeline::object::*;
    use crate::{interface::*, PipelineStartData};
    use crate::{is_render_thread, Material, Model, PipelineSendData, RenderCommand, RenderTaskReturn, Shader, SubShader, Texture, COMMAND_COUNT, RENDER_COMMAND_SENDER};
    pub use crate::{RenderTask, SharedData};
    // Start the render pipeline by initializing OpenGL on the new render thread (Ran on the main thread)
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window, barrier_data: Arc<others::WorldBarrierData>) -> PipelineStartData {
        crate::pipeline::init_pipeline(glfw, window, barrier_data)
    }
    // Join the pipeline thread and end it all
    pub fn join_pipeline(pipeline_data: PipelineStartData) {
        pipeline_data.handle.join().unwrap();
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
    }
    // Just setup the sender of commands thread-locally
    pub fn initialize_threadlocal_render_comms() {
        let pipeline = crate::pipeline::pipeline();
        let tx = pipeline.as_ref().unwrap().tx_template.clone();
        RENDER_COMMAND_SENDER.with(|sender_refcell| {
            let mut sender_ = sender_refcell.borrow_mut();
            let sender = &mut *sender_;
            *sender = Some(tx);
        });
        println!("Initialized the thread local RenderCommand sender!");
    }
    // Generate a command name
    pub fn generate_command_name() -> String {
        format!("c_{}", COMMAND_COUNT.fetch_add(1, Ordering::Relaxed))
    }
    // Normal task without callback
    pub fn task<F>(task: RenderTask, name: &str)
    {
        // Create the render command
        let command = RenderCommand {
            name: if name.is_empty() { panic!() } else { name.to_string() },
            input_task: task,
        };
        // Get the current thread ID
        let thread_id = std::thread::current().id();
        println!("\x1b[35mReceived task '{}' from thread '{}'\x1b[0m", name, std::thread::current().name().unwrap());
        // Box the callback
        let boxed_callback = Box::new(callback);
        crate::RENDER_COMMAND_SENDER.with(|x| {
            let sender_ = x.borrow();
            let sender_ = sender_.as_ref();
            let sender = sender_.unwrap();
            // Send the command to the thread
            sender.send(PipelineSendData(thread_id, command, boxed_callback, false)).unwrap();
        });
    }
    fn get_gpu_object(name: &str) -> Option<GPUObject> {
        crate::pipeline::interface::get_gpu_object(name)
    }
    pub fn gpu_object_valid(name: &str) -> bool {
        crate::pipeline::interface::gpu_object_valid(name)
    }
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
        } else if is_render_thread() {
            if let GPUObject::SubShader(x) = crate::pipeline::internal_task(RenderTask::SubShaderCreate(SharedData::new(subshader))) {
                x
            } else {
                panic!()
            }
        } else {
            let name = format!("crt_sbshdr_{}", subshader.name);
            wtask(RenderTask::SubShaderCreate(SharedData::new(subshader)), &name);
            if let GPUObject::SubShader(x) = wait_fetch_threadlocal_callbacks_specific(&name) {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn shader(shader: Shader) -> ShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_shader_object(&shader.name)
        } else if is_render_thread() {
            if let GPUObject::Shader(x) = crate::pipeline::internal_task(RenderTask::ShaderCreate(SharedData::new(shader))) {
                x
            } else {
                panic!()
            }
        } else {
            let name = format!("crt_shdr_{}", shader.name);
            wtask(RenderTask::ShaderCreate(SharedData::new(shader)), &name);
            if let GPUObject::Shader(x) = wait_fetch_threadlocal_callbacks_specific(&name) {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_compute_shader_object(&shader.name)
        } else if is_render_thread() {
            if let GPUObject::ComputeShader(x) = crate::pipeline::internal_task(RenderTask::ShaderCreate(SharedData::new(shader))) {
                x
            } else {
                panic!()
            }
        } else {
            let name = format!("crt_cmptshdr_{}", shader.name);
            wtask(RenderTask::ShaderCreate(SharedData::new(shader)), &name);
            if let GPUObject::ComputeShader(x) = wait_fetch_threadlocal_callbacks_specific(&name) {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn texture(texture: Texture) -> TextureGPUObject {
        if gpu_object_valid(&texture.name) {
            get_texture_object(&texture.name)
        } else if is_render_thread() {
            if let GPUObject::Texture(x) = crate::pipeline::internal_task(RenderTask::TextureCreate(SharedData::new(texture))) {
                x
            } else {
                panic!()
            }
        } else {
            let name = format!("crt_txtre_{}", texture.name);
            wtask(RenderTask::TextureCreate(SharedData::new(texture)), &name);
            if let GPUObject::Texture(x) = wait_fetch_threadlocal_callbacks_specific(&name) {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn model(model: Model) -> ModelGPUObject {
        // (TODO: Implement model caching)
        if is_render_thread() {
            println!("We are indeed on the render thread");
            if let GPUObject::Model(x) = crate::pipeline::internal_task(RenderTask::ModelCreate(SharedData::new(model))) {
                x
            } else {
                panic!()
            }
        } else {
            let name = format!("crt_mdl{}", model.name);
            wtask(RenderTask::ModelCreate(SharedData::new(model)), &name);
            if let GPUObject::Model(x) = wait_fetch_threadlocal_callbacks_specific(&name) {
                x
            } else {
                panic!()
            }
        }
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
