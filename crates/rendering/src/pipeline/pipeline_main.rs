use lazy_static::lazy_static;
use std::{
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{atomic::AtomicUsize, mpsc::Sender},
};

thread_local! {
    pub static IS_RENDER_THREAD: Cell<bool> = Cell::new(false);
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

    use crate::pipeline::{buffer, object::*};
    use crate::{interface, Material, Model, PipelineStartData, RenderCommandQuery, RenderCommandResult, Shader, SubShader, Texture, RENDER_COMMAND_SENDER};
    pub use crate::{RenderTask, SharedData};
    // Start the render pipeline by initializing OpenGL on the new render thread (Ran on the main thread)
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) -> PipelineStartData {
        crate::pipeline::init_pipeline(glfw, window)
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
        let tx = crate::pipeline::TX_TEMPLATE.lock().unwrap().as_ref().unwrap().clone();
        RENDER_COMMAND_SENDER.with(|sender_refcell| {
            let mut sender_ = sender_refcell.borrow_mut();
            let sender = &mut *sender_;
            *sender = Some(tx);
        });
        println!("Initialized the thread local RenderCommand sender!");
    }
    // Send a task to the render thread, returning a Command
    pub fn task(task: RenderTask) -> RenderCommandResult {
        RenderCommandResult::new(task)
    }
    // Get a GPU object
    pub fn get_gpu_object(name: &str) -> Option<GPUObject> {
        interface::get_gpu_object(name)
    }
    // Check if a GPU object is valid
    pub fn gpu_object_valid(name: &str) -> bool {
        interface::gpu_object_valid(name)
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
        } else {
            let result = task(RenderTask::SubShaderCreate(SharedData::new(subshader)));
            if let GPUObject::SubShader(x) = result.wait_gpuobject() {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn shader(shader: Shader) -> ShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_shader_object(&shader.name)
        } else {
            let result = task(RenderTask::ShaderCreate(SharedData::new(shader)));
            if let GPUObject::Shader(x) = result.wait_gpuobject() {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        if gpu_object_valid(&shader.name) {
            get_compute_shader_object(&shader.name)
        } else {
            let result = task(RenderTask::ShaderCreate(SharedData::new(shader)));
            if let GPUObject::ComputeShader(x) = result.wait_gpuobject() {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn texture(texture: Texture) -> TextureGPUObject {
        if gpu_object_valid(&texture.name) {
            get_texture_object(&texture.name)
        } else {
            let result = task(RenderTask::TextureCreate(SharedData::new(texture)));
            if let GPUObject::Texture(x) = result.wait_gpuobject() {
                x
            } else {
                panic!()
            }
        }
    }
    pub fn model(model: Model) -> ModelGPUObject {
        // (TODO: Implement model caching)
        let result = task(RenderTask::ModelCreate(SharedData::new(model)));
        if let GPUObject::Model(x) = result.wait_gpuobject() {
            x
        } else {
            panic!()
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
