use crate::{
    advanced::{atomic::AtomicGroup, compute::ComputeShader, shader_storage::ShaderStorage},
    basics::{
        lights::LightSource,
        material::Material,
        mesh::{Mesh, Vertices},
        renderer::Renderer,
        shader::{query_shader_info_tracked, Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureType},
    },
    object::{GlTracker, ObjectID, PipelineTask, ReservedTrackedID, TrackedTask},
    pipeline::{camera::Camera, pipec, sender, PipelineHandler, SceneRenderer},
    utils::{UpdateFrequency, Window, DEFAULT_WINDOW_SIZE},
};
use ahash::AHashMap;
use assets::assetc;
use glutin::NotCurrent;
use parking_lot::{Mutex, RwLock};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Barrier,
};
use veclib::{vec2, vec3};

use super::{cached::Cached, collection::Collection, defaults::DefaultPipelineObjects, settings::PipelineSettings, FrameDebugInfo, PipelineContext};

// A single pipeline callback
pub(crate) type SinglePipelineCallback = Box<dyn Fn(&mut Pipeline, &mut SceneRenderer) + Sync + Send + 'static>;
pub(crate) type PipelineEoFCallbacks = Arc<Mutex<Vec<SinglePipelineCallback>>>;

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
    pub materials: Collection<Material>,
    pub meshes: Collection<Mesh>,
    pub renderers: Collection<Renderer>,
    pub shaders: Collection<Shader>,
    pub compute_shaders: Collection<ComputeShader>,
    pub textures: Collection<Texture>,
    pub atomics: Collection<AtomicGroup>,
    pub shader_storages: Collection<ShaderStorage>,
    pub light_sources: Collection<LightSource>,

    // Store a struct that is filled with default values that we initiate at the start of the creation of this pipeline
    pub defaults: Option<DefaultPipelineObjects>,

    // Some cached values for optimization
    pub(crate) cached: Cached,

    // The current main camera that is rendering the world
    pub(crate) camera: Camera,
    // Our window
    pub window: Window,
    // Some debug info
    pub debugging: Mutex<FrameDebugInfo>,

    // End Of Frame callbacks
    pub(crate) callbacks: PipelineEoFCallbacks,

    pub time: (f64, f64),
}

impl Pipeline {
    // Set the buffered tasks from RX messages
    pub(crate) fn add_tasks(&mut self, messages: Vec<PipelineTask>) {
        let mut write = self.tasks.write();
        for task in messages {
            write.push(task);
        }
    }
    // Add a task interally, through the render thread itself
    pub(crate) fn add_task_internally(&self, task: PipelineTask) {
        let mut write = self.tasks.write();
        write.push(task);
    }
    // Execute a single tracked task, but also create a respective OpenGL fence for said task
    fn execute_tracked_task(&mut self, internal: &mut InternalPipeline, task: TrackedTask, tracking_id: ReservedTrackedID) {
        // Create a corresponding GlTracker for this task
        let gltracker = match task {
            TrackedTask::RunComputeShader(id, settings) => {
                let compute = self.compute_shaders.get(id).unwrap();
                compute.compute_run(self, settings)
            }
            TrackedTask::TextureOp(id, op) => {
                let texture = self.textures.get(id).unwrap();
                texture.buffer_operation(op)
            }
            TrackedTask::ShaderStorageOp(id, op) => {
                let shader_storage = self.shader_storages.get_mut(id).unwrap();
                shader_storage.buffer_operation(op)
            }
            TrackedTask::AtomicGroupOp(id, op) => {
                let atomic_group = self.atomics.get(id).unwrap();
                atomic_group.buffer_operation(op)
            }
            TrackedTask::QueryShaderInfo(_type, settings, read) => query_shader_info_tracked(self, _type, settings, read),
        };

        // Add the tracked ID to our pipeline
        internal.gltrackers.insert(tracking_id, gltracker);
    }
    // Execute a single task
    fn execute_task(&mut self, internal: &mut InternalPipeline, renderer: &mut SceneRenderer, task: PipelineTask) {
        // Now we must execute these tasks
        match task {
            PipelineTask::Construction(construction) => construction.execute(self),
            PipelineTask::Deconstruction(deconstruction) => deconstruction.execute(self),
            PipelineTask::Update(update) => update(self, renderer),
            PipelineTask::Tracked(task, tracking_id, _) => self.execute_tracked_task(internal, task, tracking_id),
        }
    }
    // Called each frame during the "free-zone"
    pub(crate) fn update(&mut self, internal: &mut InternalPipeline, renderer: &mut SceneRenderer) {
        // Always flush during the update
        self.flush(internal, renderer);

        // Also check each GlTracker and check if it finished executing
        let mut completed_ids: Vec<ReservedTrackedID> = Vec::new();
        for (id, tracker) in internal.gltrackers.iter_mut() {
            if tracker.completed(self) {
                completed_ids.push(*id);
            }
        }
        for id in completed_ids.iter() {
            internal.gltrackers.remove(id);
        }

        // After doing all that resetting, we can actually store the new completed IDs at their respective bitfield locations
        for completed in completed_ids {
            self.completed_tasks.set(completed.0 as usize, true);
        }
    }
    // Flush all the buffered tasks, and execute them
    // We should do this at the end of each frame, but we can force execution of it if we are running it internally
    pub(crate) fn flush(&mut self, internal: &mut InternalPipeline, renderer: &mut SceneRenderer) {
        // We must take the commands first
        let tasks = {
            let mut tasks_ = self.tasks.write();
            let tasks = &mut *tasks_;
            let taken = tasks.drain(..).collect::<Vec<_>>();
            // If we have a tracked task that requires the execution of another tracked task, we must check if the latter has completed executing
            let mut output_tasks = Vec::with_capacity(tasks.capacity());
            for task in taken {
                match task {
                    PipelineTask::Tracked(_, _, require) => {
                        // If the requirement is null, that means that the required task executed and that we can start executing the current task
                        let valid = require.and_then(|x| if self.completed_tasks.get(x.0 as usize) { None } else { Some(()) });
                        if valid.is_none() {
                            output_tasks.push(task);
                        } else {
                            tasks.push(task);
                        }
                    }
                    _ => output_tasks.push(task),
                }
            }
            output_tasks
        };

        // Execute the tasks now
        for task in tasks {
            self.execute_task(internal, renderer, task);
        }
    }
    // Run the End Of Frame callbacks
    pub(crate) fn execute_end_of_frame_callbacks(&mut self, renderer: &mut SceneRenderer) {
        let lock_ = self.callbacks.clone();
        let lock = lock_.lock();
        // Execute the callbacks
        for callback in &*lock {
            let callback = &**callback;
            callback(self, renderer);
        }
    }
    // Update methods
    // Update the window dimensions
    pub fn update_window_dimensions(&mut self, renderer: &mut SceneRenderer, new_dimensions: veclib::Vector2<u16>) {
        self.window.dimensions = new_dimensions;
        renderer.update_window_dimensions(new_dimensions, self);
    }
    // Set our internal camera to a new one
    pub fn set_internal_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
}

// Load some defaults
fn load_defaults(pipeline: &Pipeline) -> DefaultPipelineObjects {
    use assets::assetc::load_with;

    // Create the default missing texture
    let missing = pipec::construct::<Texture>(
        pipeline,
        load_with(
            "defaults/textures/missing_texture.png",
            Texture::default().with_filter(TextureFilter::Nearest).with_mipmaps(true),
        )
        .unwrap(),
    )
    .unwrap();

    // Create the default white texture
    let white = pipec::construct(
        pipeline,
        Texture::default()
            .with_dimensions(TextureType::Texture2D(1, 1))
            .with_filter(TextureFilter::Linear)
            .with_bytes(vec![255, 255, 255, 255])
            .with_mipmaps(true),
    )
    .unwrap();

    // Create the default black texture
    let black = pipec::construct(
        pipeline,
        Texture::default()
            .with_dimensions(TextureType::Texture2D(1, 1))
            .with_filter(TextureFilter::Linear)
            .with_bytes(vec![0, 0, 0, 255])
            .with_mipmaps(true),
    )
    .unwrap();

    // Create the default normal map texture
    let normals = pipec::construct(
        pipeline,
        Texture::default()
            .with_dimensions(TextureType::Texture2D(1, 1))
            .with_filter(TextureFilter::Linear)
            .with_bytes(vec![127, 127, 255, 255])
            .with_mipmaps(true),
    )
    .unwrap();

    // Create the default rendering shader
    let settings = ShaderSettings::default()
        .source("defaults/shaders/rendering/default.vrsh.glsl")
        .source("defaults/shaders/rendering/default.frsh.glsl");
    let shader = pipec::construct(pipeline, Shader::new(settings).unwrap()).unwrap();

    // Create the default material
    let material = pipec::construct(pipeline, Material::default().with_diffuse(missing)).unwrap();

    // Create the default mesh
    let mesh = pipec::construct(pipeline, Mesh::default()).unwrap();

    // Create a default plane mesh
    let plane = pipec::construct(
        pipeline,
        Mesh {
            vertices: {
                let mut vertices = Vertices::default();
                // Corners
                // TODO: Tangents
                vertices.add().with_position(vec3(-0.5, 0.0, -0.5)).with_normal(vec3(0, 127, 0)).with_uv(vec2(255, 0));
                vertices.add().with_position(vec3(0.5, 0.0, -0.5)).with_normal(vec3(0, 127, 0)).with_uv(vec2(0, 0));
                vertices.add().with_position(vec3(0.5, 0.0, 0.5)).with_normal(vec3(0, 127, 0)).with_uv(vec2(0, 255));
                vertices.add().with_position(vec3(-0.5, 0.0, 0.5)).with_normal(vec3(0, 127, 0)).with_uv(vec2(255, 255));
                vertices
            },
            update_frequency: UpdateFrequency::Static,
            indices: vec![2, 1, 0, 0, 3, 2],
            ..Default::default()
        },
    )
    .unwrap();

    // Load the default cube and sphere
    let cube = pipec::construct(pipeline, assetc::load("defaults/meshes/cube.obj").unwrap()).unwrap();
    let sphere = pipec::construct(pipeline, assetc::load("defaults/meshes/sphere.obj").unwrap()).unwrap();

    DefaultPipelineObjects {
        cube,
        sphere,
        missing_tex: missing,
        black,
        white,
        normals_tex: normals,
        shader,
        material,
        mesh,
        plane,
        sun: ObjectID::default(),
    }
}
// Initialize OpenGL
unsafe fn init_opengl() {
    gl::Viewport(0, 0, DEFAULT_WINDOW_SIZE.x as i32, DEFAULT_WINDOW_SIZE.y as i32);
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::CullFace(gl::BACK);
}
// Create the new render thread, and return some data so we can access it from other threads
pub fn init_pipeline(pipeline_settings: PipelineSettings, window: glutin::WindowedContext<NotCurrent>) -> PipelineContext {
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

    // Split
    let (gl_context, window) = unsafe { window.split() };
    // An init channel
    let (itx, irx) = std::sync::mpsc::sync_channel::<Arc<RwLock<Pipeline>>>(1);

    // Actually make the render thread
    let builder = std::thread::Builder::new().name("RenderThread".to_string());
    let handle = builder
        .spawn(move || {
            // Initialize OpenGL
            println!("Initializing OpenGL...");
            // Make the glutin context current, since we will be using the render thread for rendering
            let gl_context = unsafe { gl_context.make_current().unwrap() };
            gl::load_with(|x| gl_context.get_proc_address(x));
            // Check if the gl viewport is ok
            if gl::Viewport::is_loaded() {
                unsafe {
                    init_opengl();
                }
            } else {
                panic!()
            }
            println!("Successfully initialized OpenGL!");

            // Set the global sender
            sender::set_global_sender(tx);

            // Create the pipeline
            let pipeline = Pipeline::default();
            // Create an internal pipeline as well
            let mut internal = InternalPipeline::default();

            // Create the Arc and RwLock for the pipeline
            let pipeline = Arc::new(RwLock::new(pipeline));

            let mut pipeline_ = pipeline.write();
            // Setup the window
            pipeline_.window.pixels_per_point = window.scale_factor();
            pipeline_.window.inner = Some(window);

            // Load the default objects
            pipeline_.defaults = Some(load_defaults(&pipeline_));
            drop(pipeline_);
            // Setup the pipeline renderer
            let mut renderer = {
                let mut pipeline = pipeline.write();
                let mut renderer = SceneRenderer::default();
                pipeline.flush(&mut internal, &mut renderer);
                renderer.initialize(pipeline_settings, &mut internal, &mut *pipeline);
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
                let pipeline_frame_instant = std::time::Instant::now();
                let mut pipeline_ = pipeline.write();
                let time = time_clone.lock();
                pipeline_.time = *time;

                drop(time);
                drop(pipeline_);

                let i = std::time::Instant::now();
                // We render the scene here
                let pipeline_ = pipeline.read();
                let frame_debug_info = renderer.render_frame(&*pipeline_);
                let render_frame_duration = i.elapsed();
                // And we also sync at the end of each frame
                ebarrier_clone.wait();
                drop(pipeline_);

                // This is the "free-zone". A time between the end barrier sync and the start barrier sync where we can do whatever we want with the pipeline
                let mut pipeline = pipeline.write(); // We poll the messages, buffer them, and execute them
                let i = std::time::Instant::now();
                pipeline.execute_end_of_frame_callbacks(&mut renderer);
                let eof_callbacks_duration = i.elapsed();

                // Do not forget to switch buffers at the end of the frame
                let i = std::time::Instant::now();
                gl_context.swap_buffers().unwrap();
                let swap_buffers_duration = i.elapsed();

                let messages = rx.try_iter().collect::<Vec<PipelineTask>>();
                // Set the buffer
                pipeline.add_tasks(messages);
                // Execute the tasks
                pipeline.update(&mut internal, &mut renderer);

                // Set the debug info
                let mut debug = pipeline.debugging.lock();
                *debug = frame_debug_info;
                debug.whole_frame = pipeline_frame_instant.elapsed().as_secs_f32() * 1000.0;
                debug.render_frame = render_frame_duration.as_secs_f32() * 1000.0;
                debug.eof_callbacks_execution = eof_callbacks_duration.as_secs_f32() * 1000.0;
                debug.swap_buffers = swap_buffers_duration.as_secs_f32() * 1000.0;

                // Check if we must exit from the render thread
                if eatomic_clone.load(Ordering::Relaxed) {
                    break;
                }
            }
            println!("Stopped the render thread!");
        })
        .unwrap();
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    let pipeline = irx.recv().unwrap();
    println!("Successfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());
    // Create the pipeline context
    PipelineContext {
        pipeline,
        handler: Some(Arc::new(Mutex::new(PipelineHandler {
            handle,
            sbarrier,
            ebarrier,
            eatomic,
            waiting,
            time,
        }))),
    }
}
