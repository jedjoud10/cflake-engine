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

pub fn rname(prefix: &str) -> String {
    // Use the others::id_counter to create a counted ID that we can transform into a String
    let name = format!("{:x}", others::id_counter::get_id());
    format!("{}_{}", prefix, name)
}

pub mod pipec {
    use assets::CachedObject;
    use std::borrow::{Borrow, BorrowMut};

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, MutexGuard};

    pub use super::super::others;
    use crate::pipeline::batch_command::BatchRenderCommandQueryResult;
    use crate::pipeline::buffer::PipelineBuffer;
    use crate::pipeline::{buffer, object::*};
    pub use crate::RenderTask;
    use crate::{
        is_render_thread, GPUObjectID, Material, Model, PipelineStartData, RenderCommandQuery, RenderCommandQueryResult, Shader, SubShader, Texture, RENDER_COMMAND_SENDER,
    };
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
    // Execute a RenderCommandQueryResult. This runs it internally if we are on the render thread.
    fn execute(rs: RenderCommandQueryResult) -> GPUObjectID {
        if is_render_thread() {
            let mut buf = crate::pipeline::pipeline::BUFFER.lock().unwrap();
            rs.wait_internal(&mut buf)
        } else {
            rs.wait()
        }
    }
    // Send a task to the render thread, returning a Command
    pub fn task(task: RenderTask) -> RenderCommandQueryResult {
        RenderCommandQueryResult::new(task)
    }
    // Create a batch of tasks
    pub fn task_batch(results: Vec<RenderCommandQueryResult>) -> BatchRenderCommandQueryResult {
        // Extract the tasks from the command query results
        let tasks = results.into_iter().map(|mut x| x.task.take().unwrap()).collect::<Vec<RenderTask>>();
        BatchRenderCommandQueryResult::new(tasks)
    }
    // Load or create functions
    pub fn subshader(subshader: SubShader) -> GPUObjectID {
        execute(match others::get_id(&subshader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::SubShaderCreate(subshader)),
        })
    }
    pub fn shader(shader: Shader) -> GPUObjectID {
        execute(match others::get_id(&shader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(shader)),
        })
    }
    pub fn compute_shader(shader: Shader) -> GPUObjectID {
        execute(match others::get_id(&shader.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(shader)),
        })
    }
    pub fn texture(texture: Texture) -> GPUObjectID {
        execute(match others::get_id(&texture.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::TextureCreate(texture)),
        })
    }
    pub fn model(model: Model) -> GPUObjectID {
        execute(match others::get_id(&model.name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ModelCreate(model)),
        })
    }
    pub fn material(material: Material) -> GPUObjectID {
        execute(match others::get_id(&material.material_name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::MaterialCreate(material)),
        })
    }
    pub fn uniforms(uniforms: ShaderUniformsGroup) -> GPUObjectID {
        execute(task(RenderTask::UniformsCreate(uniforms)))
    }
    // Load or create functions, cached type
    pub fn texturec(texturec: CachedObject<Texture>) -> GPUObjectID {
        execute(match others::get_id(&texturec.arc.as_ref().name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::TextureCreate(texturec.arc.as_ref().clone())),
        })
    }
    pub fn shaderc(shaderc: CachedObject<Shader>) -> GPUObjectID {
        execute(match others::get_id(&shaderc.arc.as_ref().name) {
            Some(id) => RenderCommandQueryResult::new_id(id),
            None => task(RenderTask::ShaderCreate(shaderc.arc.as_ref().clone())),
        })
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
