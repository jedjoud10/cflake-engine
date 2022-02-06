use crate::{
    advanced::{atomic::AtomicGroup, compute::ComputeShader, shader_storage::ShaderStorage},
    basics::{
        lights::LightSource,
        material::Material,
        model::{Model, ModelBuffers},
        renderer::Renderer,
        shader::{query_shader_info, Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureType},
        uniforms::{ShaderIdentifier, ShaderUniformsGroup, ShaderUniformsSettings},
    },
    object::{GlTracker, ObjectID, PipelineTask, ReservedTrackedID, TrackedTask},
    pipeline::{camera::Camera, pipec, sender, PipelineHandler, PipelineRenderer},
    utils::{RenderWrapper, Window},
};
use ahash::AHashMap;
use glfw::Context;
use ordered_vec::shareable::ShareableOrderedVec;
use std::{
    ptr::null_mut,
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc, Barrier, Mutex, RwLock,
    },
};

use super::PipelineContext;

// Some default values like the default material or even the default shader
pub struct DefaultPipelineObjects {
    pub missing_tex: ObjectID<Texture>,
    pub black: ObjectID<Texture>,
    pub white: ObjectID<Texture>,
    pub normals_tex: ObjectID<Texture>,
    pub shader: ObjectID<Shader>,
    pub material: ObjectID<Material>,
    pub model: ObjectID<Model>,
    // This value might change, since the user might remove the directional light
    pub sun: ObjectID<LightSource>,
}

// Some internal pipeline data that we store on the render thread and that we cannot share with the other threads
#[derive(Default)]
pub(crate) struct InternalPipeline {
    // Keep track of the GlTrackers and their corresponding ID
    gltrackers: AHashMap<ReservedTrackedID, GlTracker>,
}

// The rendering pipeline. It can be shared around using Arc, but we are only allowed to modify it on the Render Thread
// This is only updated at the end of each frame, so we don't have to worry about reading it from multiple threads since no one will be writing to it at that times
#[derive(Default)]
pub struct Pipeline {
    // We will buffer the tasks, so that way whenever we receive a task internally from the Render Thread itself we can just wait until we manually flush the tasks to execute all at once
    tasks: RwLock<Vec<PipelineTask>>,

    // Tracked tasks
    pub(crate) completed_tasks: bitfield::AtomicSparseBitfield,

    // We store the Pipeline Objects, for each Pipeline Object type
    // We will create these Pipeline Objects *after* they have been created by OpenGL (if applicable)
    pub(crate) materials: ShareableOrderedVec<Material>,
    pub(crate) models: ShareableOrderedVec<(Model, ModelBuffers)>,
    pub(crate) renderers: ShareableOrderedVec<Renderer>,
    pub(crate) shaders: ShareableOrderedVec<Shader>,
    pub(crate) compute_shaders: ShareableOrderedVec<ComputeShader>,
    pub(crate) textures: ShareableOrderedVec<Texture>,
    pub(crate) atomics: ShareableOrderedVec<AtomicGroup>,
    pub(crate) shader_storages: ShareableOrderedVec<ShaderStorage>,
    pub(crate) light_sources: ShareableOrderedVec<LightSource>,

    // Store a struct that is filled with default values that we initiate at the start of the creation of this pipeline
    pub defaults: Option<DefaultPipelineObjects>,

    // The current main camera that is rendering the world
    pub(crate) camera: Camera,
    // Our window
    pub window: Window,
    // Atomic used to debug some data
    pub(crate) debugging: AtomicBool,

    // End Of Frame callbacks
    pub(crate) callbacks: Arc<Mutex<Vec<Box<dyn Fn(&mut Pipeline, &mut PipelineRenderer) + Sync + Send + 'static>>>>,

    pub time: (f64, f64),
}

impl Pipeline {
    // Set the buffered tasks from RX messages
    pub(crate) fn add_tasks(&mut self, messages: Vec<PipelineTask>) {
        let mut write = self.tasks.write().unwrap();
        for task in messages {
            write.push(task);
        }
    }
    // Add a task interally, through the render thread itself
    pub(crate) fn add_task_internally(&self, task: PipelineTask) {
        let mut write = self.tasks.write().unwrap();
        write.push(task);
    }
    // Execute a single tracked task, but also create a respective OpenGL fence for said task
    fn execute_tracked_task(&mut self, internal: &mut InternalPipeline, task: TrackedTask, tracking_id: ReservedTrackedID) {
        // Create a corresponding GlTracker for this task
        let gltracker = match task {
            TrackedTask::RunComputeShader(id, settings) => {
                let compute = self.get_compute_shader(id).unwrap();
                compute.compute_run(self, settings)
            }
            TrackedTask::TextureReadBytes(id, read) => {
                let texture = self.get_texture(id).unwrap();
                texture.read_bytes(self, read)
            }
            TrackedTask::ShaderStorageReadBytes(id, read) => {
                let shader_storage = self.get_shader_storage(id).unwrap();
                shader_storage.read_bytes(self, read)
            }
            TrackedTask::AtomicGroupRead(id, read) => {
                let atomic_group = self.get_atomic_group(id).unwrap();
                atomic_group.read_counters(self, read)
            }
            TrackedTask::QueryShaderInfo(id, settings, read) => {
                let shader = self.get_shader(id).unwrap();
                query_shader_info(self, shader.program, settings, read)
            }
            TrackedTask::QueryComputeShaderInfo(id, settings, read) => {
                let compute = self.get_compute_shader(id).unwrap();
                query_shader_info(self, compute.program, settings, read)
            }
        };

        // Add the tracked ID to our pipeline
        internal.gltrackers.insert(tracking_id, gltracker);

        // Also check each GlTracker and check if it finished executing
        let completed_ids = internal.gltrackers.drain_filter(|_id, tracker| tracker.completed(self)).collect::<Vec<_>>();

        // After doing all that resetting, we can actually store the new completed IDs at their respective bitfield locations
        for (completed, _) in completed_ids {
            self.completed_tasks.set(completed.0, true);
        }
    }
    // Execute a single task
    fn execute_task(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer, task: PipelineTask) {
        // Now we must execute these tasks
        match task {
            PipelineTask::Construction(construction) => construction.execute(self),
            PipelineTask::Deconstruction(deconstruction) => deconstruction.execute(self),
            PipelineTask::Update(update) => update(self, renderer),
            PipelineTask::Tracked(task, tracking_id, _) => self.execute_tracked_task(internal, task, tracking_id),
        }
    }
    // Called each frame during the "free-zone"
    pub(crate) fn update(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer) {
        // Also check each GlTracker and check if it finished executing
        let completed_ids = internal.gltrackers.drain_filter(|_id, tracker| tracker.completed(self)).collect::<Vec<_>>();

        // After doing all that resetting, we can actually store the new completed IDs at their respective bitfield locations
        for (completed, _) in completed_ids {
            self.completed_tasks.set(completed.0, true);
        }

        // Always flush during the update
        self.flush(internal, renderer);
    }
    // Flush all the buffered tasks, and execute them
    // We should do this at the end of each frame, but we can force execution of it if we are running it internally
    pub(crate) fn flush(&mut self, internal: &mut InternalPipeline, renderer: &mut PipelineRenderer) {
        // We must take the commands first
        let tasks = {
            let mut tasks_ = self.tasks.write().unwrap();
            let tasks = &mut *tasks_;
            // If we have a tracked task that requires the execution of another tracked task, we must check if the latter has completed executing
            let tasks = tasks
                .drain_filter(|task| match task {
                    PipelineTask::Tracked(_, _, require) => {
                        // If the requirement is null, that means that we don't need it
                        let valid = require.and_then(|x| if self.completed_tasks.get(x.0) { None } else { Some(()) });
                        valid.is_none()
                    }
                    _ => true,
                })
                .collect::<Vec<_>>();
            tasks
        };

        // Execute the tasks now
        for task in tasks {
            self.execute_task(internal, renderer, task);
        }

        // Update the window if needed
        let update_window = self.window.update.load(Ordering::Relaxed);
        if update_window {
            let (glfw, window) = (self.window.wrapper.0.load(Ordering::Relaxed), self.window.wrapper.1.load(Ordering::Relaxed));
            let (glfw, _window) = unsafe { (&mut *glfw, &mut *window) };
            if self.window.vsync.load(Ordering::Relaxed) {
                glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            } else {
                glfw.set_swap_interval(glfw::SwapInterval::None);
            }
        }
    }
    // Set the global shader uniforms
    pub(crate) fn update_global_shader_uniforms(&mut self, time: f64, delta: f64) {
        for (_id, shader) in self.shaders.iter() {
            // Set the uniforms
            let mut group = ShaderUniformsGroup::new();
            group.set_f32("_time", time as f32);
            group.set_f32("_delta", delta as f32);
            group.set_vec2i32("_resolution", self.window.dimensions.into());
            let id = ShaderUniformsSettings::new(ShaderIdentifier::OpenGLID(shader.program));
            group.execute(self, id).unwrap();
        }
    }
    // Run the End Of Frame callbacks
    pub(crate) fn execute_end_of_frame_callbacks(&mut self, renderer: &mut PipelineRenderer) {
        let lock_ = self.callbacks.clone();
        let lock = lock_.lock().unwrap();
        // Execute the callbacks
        for callback in &*lock {
            let callback = &**callback;
            callback(self, renderer);
        }
    }
    // Get a material using it's respective ID
    pub fn get_material(&self, id: ObjectID<Material>) -> Option<&Material> {
        self.materials.get(id.get()?)
    }
    // Get a model using it's respective ID
    pub fn get_model(&self, id: ObjectID<Model>) -> Option<&(Model, ModelBuffers)> {
        self.models.get(id.get()?)
    }
    // Get a renderer using it's respective ID
    pub fn get_renderer(&self, id: ObjectID<Renderer>) -> Option<&Renderer> {
        self.renderers.get(id.get()?)
    }
    // Get a shader using it's respective ID
    pub fn get_shader(&self, id: ObjectID<Shader>) -> Option<&Shader> {
        self.shaders.get(id.get()?)
    }
    // Get a compute shader using it's respective ID
    pub fn get_compute_shader(&self, id: ObjectID<ComputeShader>) -> Option<&ComputeShader> {
        self.compute_shaders.get(id.get()?)
    }
    // Get a texture using it's respective ID
    pub fn get_texture(&self, id: ObjectID<Texture>) -> Option<&Texture> {
        self.textures.get(id.get()?)
    }
    // Get an atomic group using it's respective ID
    pub fn get_atomic_group(&self, id: ObjectID<AtomicGroup>) -> Option<&AtomicGroup> {
        self.atomics.get(id.get()?)
    }
    // Get a shader storage using it's respective ID
    pub fn get_shader_storage(&self, id: ObjectID<ShaderStorage>) -> Option<&ShaderStorage> {
        self.shader_storages.get(id.get()?)
    }
    // Get a light source using it's repsective ID
    pub fn get_light_source(&self, id: ObjectID<LightSource>) -> Option<&LightSource> {
        self.light_sources.get(id.get()?)
    }

    // Mutable
    // Get a mutable material using it's respective ID
    pub fn get_material_mut(&mut self, id: ObjectID<Material>) -> Option<&mut Material> {
        self.materials.get_mut(id.get()?)
    }
    // Get a mutable model using it's respective ID
    pub fn get_model_mut(&mut self, id: ObjectID<Model>) -> Option<&mut (Model, ModelBuffers)> {
        self.models.get_mut(id.get()?)
    }
    // Get a mutable renderer using it's respective ID
    pub fn get_renderer_mut(&mut self, id: ObjectID<Renderer>) -> Option<&mut Renderer> {
        self.renderers.get_mut(id.get()?)
    }
    // Get a mutable shader using it's respective ID
    pub fn get_shader_mut(&mut self, id: ObjectID<Shader>) -> Option<&mut Shader> {
        self.shaders.get_mut(id.get()?)
    }
    // Get a mutable compute shader using it's respective ID
    pub fn get_compute_shader_mut(&mut self, id: ObjectID<ComputeShader>) -> Option<&mut ComputeShader> {
        self.compute_shaders.get_mut(id.get()?)
    }
    // Get a mutable texture using it's respective ID
    pub fn get_texture_mut(&mut self, id: ObjectID<Texture>) -> Option<&mut Texture> {
        self.textures.get_mut(id.get()?)
    }
    // Get a mutable atomic group using it's respective ID
    pub fn get_atomic_group_mut(&mut self, id: ObjectID<AtomicGroup>) -> Option<&mut AtomicGroup> {
        self.atomics.get_mut(id.get()?)
    }
    // Get a mutable shader storage using it's respective ID
    pub fn get_shader_storage_mut(&mut self, id: ObjectID<ShaderStorage>) -> Option<&mut ShaderStorage> {
        self.shader_storages.get_mut(id.get()?)
    }
    // Get a mutable light source using it's repsective ID
    pub fn get_light_source_mut(&mut self, id: ObjectID<LightSource>) -> Option<&mut LightSource> {
        self.light_sources.get_mut(id.get()?)
    }

    // Update methods
    // Update the window dimensions
    pub fn update_window_dimensions(&mut self, renderer: &mut PipelineRenderer, new_dimensions: veclib::Vector2<u16>) {
        self.window.dimensions = new_dimensions;
        renderer.update_window_dimensions(new_dimensions, self);
    }
    // Update the focus state for our window
    pub fn update_window_focus_state(&mut self, focused: bool) {
        self.window.focused = focused;
    }
    // Set our internal camera to a new one
    pub fn set_internal_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
}

// Load some defaults
fn load_defaults(pipeline: &Pipeline) -> DefaultPipelineObjects {
    use assets::assetc::load;

    // Create the default missing texture
    let missing = pipec::construct::<Texture>(pipeline, load("defaults\\textures\\missing_texture.png", Texture::default().set_mipmaps(true)).unwrap()).unwrap();

    // Create the default white texture
    let white = pipec::construct(
        pipeline,
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![255, 255, 255, 255])
            .set_mipmaps(true),
    )
    .unwrap();

    // Create the default black texture
    let black = pipec::construct(
        pipeline,
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![0, 0, 0, 255])
            .set_mipmaps(true),
    )
    .unwrap();

    // Create the default normal map texture
    let normals = pipec::construct(
        pipeline,
        Texture::default()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .set_bytes(vec![127, 128, 255, 255])
            .set_mipmaps(true),
    )
    .unwrap();

    // Create the default rendering shader
    let settings = ShaderSettings::default()
        .source("defaults\\shaders\\rendering\\default.vrsh.glsl")
        .source("defaults\\shaders\\rendering\\default.frsh.glsl");
    let shader = pipec::construct(pipeline, Shader::new(settings).unwrap()).unwrap();

    // Create the default material
    let material = pipec::construct(pipeline, Material::default()).unwrap();

    // Create the default model
    let model = pipec::construct(pipeline, Model::default()).unwrap();

    DefaultPipelineObjects {
        missing_tex: missing,
        black,
        white,
        normals_tex: normals,
        shader,
        material,
        model,
        sun: ObjectID::default(),
    }
}
// Initialize OpenGL
unsafe fn init_opengl() {
    gl::Viewport(0, 0, 1280, 720);
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::CullFace(gl::BACK);
}
// Create the new render thread, and return some data so we can access it from other threads
pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) -> PipelineContext {
    println!("Initializing RenderPipeline...");
    // Create a single channel to allow us to receive Pipeline Tasks from the other threads
    let (tx, rx) = std::sync::mpsc::channel::<PipelineTask>(); // Main to render

    // Barrier so we can sync with the main thread at the start of each frame
    let sbarrier = Arc::new(Barrier::new(2));
    let sbarrier_clone = sbarrier.clone();

    // Barrier so we can sync with the main thread at the end of each frame
    let ebarrier = Arc::new(Barrier::new(2));
    let ebarrier_clone = ebarrier.clone();

    // An atomic that we use to inform the render thread to exit and shutdown
    let eatomic = Arc::new(AtomicBool::new(false));
    let eatomic_clone = eatomic.clone();

    // Waiting
    let waiting = Arc::new(AtomicBool::new(false));
    let waiting_clone = waiting.clone();

    // Some timing data that we will share with the pipeline
    let time = Arc::new(Mutex::new((0.0, 0.0)));
    let time_clone = time.clone();

    // An init channel
    let (itx, irx) = std::sync::mpsc::sync_channel::<Arc<RwLock<Pipeline>>>(1);

    // The main thread will not own the glfw context, and we will send it to the render thread instead
    unsafe { glfw::ffi::glfwMakeContextCurrent(null_mut()) }

    // Window and GLFW wrapper
    let wrapper = {
        // Create the render wrapper
        let glfw = glfw as *mut glfw::Glfw;
        let window = window as *mut glfw::Window;
        Arc::new(RenderWrapper(AtomicPtr::new(glfw), AtomicPtr::new(window)))
    };

    // Actually make the render thread
    let handle = std::thread::spawn(move || {
        // Start OpenGL
        let _glfw = unsafe { &mut *wrapper.0.load(std::sync::atomic::Ordering::Relaxed) };
        let window = unsafe { &mut *wrapper.1.load(std::sync::atomic::Ordering::Relaxed) };
        // Initialize OpenGL
        println!("Initializing OpenGL...");
        window.make_current();
        unsafe {
            glfw::ffi::glfwMakeContextCurrent(window.window_ptr());
            gl::load_with(|s| window.get_proc_address(s) as *const _);
        }
        if gl::Viewport::is_loaded() {
            unsafe {
                init_opengl();
            }
            println!("Successfully initialized OpenGL!");
        } else {
            /* NON */
            panic!()
        }
        // Window wrapper
        let window_wrapper = Window::new(wrapper.clone());

        // Set the global sender
        sender::set_global_sender(tx);

        // Create the pipeline
        let pipeline = Pipeline::default();
        // Create an internal pipeline as well
        let mut internal = InternalPipeline::default();

        // Create the Arc and RwLock for the pipeline
        let pipeline = Arc::new(RwLock::new(pipeline));

        // Load the default objects
        {
            let mut pipeline = pipeline.write().unwrap();
            pipeline.window = window_wrapper;
            pipeline.defaults = Some(load_defaults(&*pipeline));
        }

        // Setup the pipeline renderer
        let mut renderer = {
            let mut pipeline = pipeline.write().unwrap();
            let mut renderer = PipelineRenderer::default();
            pipeline.flush(&mut internal, &mut renderer);
            renderer.initialize(&mut internal, &mut *pipeline);
            renderer
        };

        // ---- Finished initializing the Pipeline! ----
        itx.send(pipeline.clone()).unwrap();
        println!("Successfully created the RenderThread!");

        // We must render every frame
        loop {
            // At the start of each frame we must sync up with the main thread
            waiting_clone.store(true, Ordering::Relaxed);
            sbarrier_clone.wait();
            waiting_clone.store(false, Ordering::Relaxed);
            // This is a single frame
            let mut pipeline_ = pipeline.write().unwrap();
            let time = time_clone.lock().unwrap();
            pipeline_.update_global_shader_uniforms(time.0, time.1);
            pipeline_.time = *time;
            let debug = pipeline_.debugging.load(Ordering::Relaxed);
            
            drop(time);
            drop(pipeline_);

            let i = std::time::Instant::now();
            // We render the scene here
            let pipeline_ = pipeline.read().unwrap();
            renderer.render_frame(&*pipeline_);
            let render_frame_duration = i.elapsed();
            // And we also sync at the end of each frame
            ebarrier_clone.wait();
            drop(pipeline_);

            // This is the "free-zone". A time between the end barrier sync and the start barrier sync where we can do whatever we want with the pipeline
            let mut pipeline = pipeline.write().unwrap(); // We poll the messages, buffer them, and execute them
            let i = std::time::Instant::now();
            pipeline.execute_end_of_frame_callbacks(&mut renderer);
            let eof_callbacks_duration = i.elapsed();
            
            // Do not forget to switch buffers at the end of the frame
            window.swap_buffers();

            let i = std::time::Instant::now();
            let messages = rx.try_iter().collect::<Vec<PipelineTask>>();
            // Set the buffer
            pipeline.add_tasks(messages);            
            // Execute the tasks
            pipeline.update(&mut internal, &mut renderer);
            let update_duration = i.elapsed();
            
            // Debug if needed
            if debug {
                println!("Pipeline: ");
                println!("  #Pipeline Render Frame Time: {:.2}ms", render_frame_duration.as_secs_f32() * 1000.0);
                println!("  #Pipeline EoF Callbacks Execution Time: {:.2}ms", eof_callbacks_duration.as_secs_f32() * 1000.0);
                println!("  #Pipeline Update Execution Time: {:.2}ms", update_duration.as_secs_f32() * 1000.0);
            }

            // Check if we must exit from the render thread
            if eatomic_clone.load(Ordering::Relaxed) {
                break;
            }
        }
        println!("Stopped the render thread!");
    });
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    let pipeline = irx.recv().unwrap();
    println!("Successfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());
    // Create the pipeline context
    PipelineContext {
        pipeline,
        handler: Arc::new(Mutex::new(PipelineHandler {
            handle,
            sbarrier,
            ebarrier,
            eatomic,
            waiting,
            time,
        })),
    }
}
