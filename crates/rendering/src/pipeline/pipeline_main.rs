use lazy_static::lazy_static;
use std::{
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{atomic::AtomicUsize, mpsc::Sender},
};

thread_local! {
    pub(crate) static IS_RENDER_THREAD: Cell<bool> = Cell::new(false);
}
// Check if we are on the render thread
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

    use crate::pipeline::{buffer, object::*};
    use crate::{GPUObjectID, Material, Model, PipelineStartData, RenderCommandQuery, RenderCommandQueryResult, Shader, SubShader, Texture, RENDER_COMMAND_SENDER, is_render_thread};
    pub use crate::{RenderTask, SharedData};
    pub use super::super::others as others;
    // Start the render pipeline by initializing OpenGL on the new render thread (Ran on the main thread)
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) -> PipelineStartData {
        crate::pipeline::init_pipeline(glfw, window)
    }
    // Join the pipeline thread and end it all
    pub fn join_pipeline(pipeline_data: PipelineStartData) {
        pipeline_data.handle.join().unwrap();
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
    pub fn task(task: RenderTask) -> RenderCommandQueryResult {
        RenderCommandQueryResult::new(task)
    }
    // Load or create functions
    pub fn subshader(subshader: SubShader) -> RenderCommandQueryResult {
        match others::get_id(&subshader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::SubShaderCreate(SharedData::new(subshader))),
        } 
    }
    pub fn shader(shader: Shader) -> RenderCommandQueryResult {
        match others::get_id(&shader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(SharedData::new(shader))),
        } 
    }
    pub fn compute_shader(shader: Shader) -> RenderCommandQueryResult {
        match others::get_id(&shader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(SharedData::new(shader))),
        }
    }
    pub fn texture(texture: Texture) -> RenderCommandQueryResult {
        match others::get_id(&texture.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::TextureCreate(SharedData::new(texture))),
        }
    }
    pub fn model(model: Model) -> RenderCommandQueryResult {
        // (TODO: Implement model caching)
        task(RenderTask::ModelCreate(SharedData::new(model)))
    }
    pub fn material(material: Material) -> RenderCommandQueryResult {
        match others::get_id(&material.material_name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::MaterialCreate(SharedData::new(material))),
        }
    }
    // Load or create functions, cached type
    pub fn texturec(texturec: CachedObject<Texture>) -> RenderCommandQueryResult {        
        match others::get_id(&texturec.arc.as_ref().name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::TextureCreate(SharedData::new(texturec.arc.as_ref().clone()))),
        }
    }
    pub fn shaderc(shaderc: CachedObject<Shader>) -> RenderCommandQueryResult {
        match others::get_id(&shaderc.arc.as_ref().name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(SharedData::new(shaderc.arc.as_ref().clone()))),
        }
    }
    // Read the data from an array that was filled using a texture
    pub fn convert_native<T>(vec: Vec<u8>) -> Vec<T>
    where
        T: Default + Clone + Sized,
    {
        // Unsafe
        let t = vec.chunks_exact(std::mem::size_of::<T>()).map(|x| unsafe { std::ptr::read::<T>(x.as_ptr() as *const _) });
        let pixels: Vec<T> = t.collect();
        pixels
    }
    pub fn convert_native_veclib<T, U>(vec: Vec<u8>) -> Vec<T>
    where
        T: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Unsafe
        let t = vec.chunks_exact(std::mem::size_of::<T>()).map(|x| unsafe { std::ptr::read::<T>(x.as_ptr() as *const _) });
        let pixels: Vec<T> = t.collect();
        pixels
    }
}
