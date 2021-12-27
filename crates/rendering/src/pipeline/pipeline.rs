use super::{object::*, async_command_data::AsyncGPUCommandData};
use crate::{basics::*, rendering::PipelineRenderer, GPUObjectID, RenderCommandQuery, RenderTask, SharedData, pipeline::buffer::PipelineBuffer, others::{RESULT, CommandExecutionResults}};
use glfw::Context;
use lazy_static::lazy_static;
use others::SmartList;
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        mpsc::{Receiver, Sender},
        Arc, Barrier, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
    }, cell::RefCell,
};

// Messages that will be to the main thread
pub enum MainThreadMessage {
    ExecuteGPUObjectCallback(u64, (GPUObject, GPUObjectID), std::thread::ThreadId),
    ExecuteExecutionCallback(u64, std::thread::ThreadId),
}

lazy_static! {
    // Template render command query sender that we can copy over the multiple task threads
    pub static ref TX_TEMPLATE: Mutex<Option<Sender<RenderCommandQuery>>> = Mutex::new(None);

    // This might be in a mutex, but we never share it around the threads. This is only a static because I don't want to manually implement internal functions for internal render thread commands
    pub(crate) static ref BUFFER: Mutex<PipelineBuffer> = Mutex::new(PipelineBuffer::default());
}

// Load the default rendering things
fn load_defaults() {
    crate::pipec::texturec(assets::cachec::acache_l("defaults\\textures\\missing_texture.png", Texture::default().enable_mipmaps()).unwrap());
    // Create the black texture
    crate::pipec::texturec(
        assets::cachec::cache(
            "black",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("black")
                .set_bytes(vec![0, 0, 0, 255]),
        )
        .unwrap(),
    );

    // Create the white texture
    crate::pipec::texturec(
        assets::cachec::cache(
            "white",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("white")
                .set_bytes(vec![255, 255, 255, 255]),
        )
        .unwrap(),
    );
    // Create the default normals texture
    crate::pipec::texturec(
        assets::cachec::cache(
            "default_normals",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("default_normals")
                .set_bytes(vec![127, 128, 255, 255]),
        )
        .unwrap(),
    );
}

// Create the new render thread
pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) -> PipelineStartData {
    println!("Initializing RenderPipeline...");
    // Create a single channel (WorkerThreads/MainThread  => Render Thread)
    let (tx, rx) = std::sync::mpsc::channel::<RenderCommandQuery>(); // Main to render
    let (tx2, rx2) = std::sync::mpsc::channel::<MainThreadMessage>(); // Render to main
    {
        let mut template_ = TX_TEMPLATE.lock().unwrap();
        let template = &mut *template_;
        *template = Some(tx);
    }
    // Barrier so we can wait until the render thread has finished initializing
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();
    let join_handle: std::thread::JoinHandle<()>;
    unsafe {
        // Window and GLFW wrapper
        struct RenderWrapper(AtomicPtr<glfw::Glfw>, AtomicPtr<glfw::Window>);
        // Create the render wrapper
        let glfw = glfw as *mut glfw::Glfw;
        let window = window as *mut glfw::Window;
        let render_wrapper = RenderWrapper(AtomicPtr::new(glfw), AtomicPtr::new(window));
        unsafe impl Send for RenderWrapper {}
        unsafe impl Sync for RenderWrapper {}
        let builder = std::thread::Builder::new().name("RenderThread".to_string());
        join_handle = builder
            .spawn(move || {
                // Start OpenGL
                let glfw = &mut *render_wrapper.0.load(std::sync::atomic::Ordering::Relaxed);
                let window = &mut *render_wrapper.1.load(std::sync::atomic::Ordering::Relaxed);
                // Initialize OpenGL
                println!("Initializing OpenGL...");
                window.make_current();
                glfw::ffi::glfwMakeContextCurrent(window.window_ptr());
                gl::load_with(|s| window.get_proc_address(s) as *const _);

                // Set the type of events that we want to listen to
                window.set_key_polling(true);
                window.set_cursor_pos_polling(true);
                window.set_scroll_polling(true);
                window.set_size_polling(true);
                glfw.set_swap_interval(glfw::SwapInterval::None);
                window.make_current();
                if gl::Viewport::is_loaded() {
                    gl::Viewport(0, 0, 1280, 720);
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Enable(gl::DEPTH_TEST);
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::BACK);
                    println!("Successfully initialized OpenGL!");
                } else {
                    /* NON */
                    panic!()
                }
                // The render command receiver
                let sent_tasks_receiver = rx;

                // Create the pipeline
                let mut pipeline_renderer = PipelineRenderer::default();
                // This is indeed the render thread
                crate::pipeline::IS_RENDER_THREAD.with(|x| x.set(true));
                load_defaults();
                pipeline_renderer.init();

                // El camera
                let mut camera = CameraDataGPUObject {
                    position: veclib::Vector3::ZERO,
                    rotation: veclib::Quaternion::IDENTITY,
                    clip_planes: veclib::Vector2::ZERO,
                    viewm: veclib::Matrix4x4::IDENTITY,
                    projm: veclib::Matrix4x4::IDENTITY,
                };

                // We must render every frame
                // Timing stuff
                let mut last_time: f64 = 0.0;
                let mut frame_count: u128 = 0;
                let tx2 = tx2.clone();
                println!("Successfully created the RenderThread!");
                barrier_clone.wait();
                
                loop {
                    // Update the delta_time                    
                    let new_time = glfw.get_time();
                    let delta = new_time - last_time;
                    last_time = new_time;
                    // Run the frame
                    // Poll first
                    let mut pipeline_buffer = BUFFER.lock().unwrap();
                    poll_commands(&mut pipeline_buffer, &mut pipeline_renderer, &mut camera, &sent_tasks_receiver, window, glfw);
                    poll_async_gpu_commands(&mut pipeline_buffer);
                    // --- Rendering ---
                    // Pre-render
                    pipeline_renderer.pre_render();
                    // Render
                    pipeline_renderer.renderer_frame(&mut pipeline_buffer, &camera);
                    // Post-render
                    pipeline_renderer.post_render(&pipeline_buffer, &camera, window);

                    // Run the callbacks
                    pipeline_buffer.execute_callbacks(&tx2);

                    frame_count += 1;
                    // The world is valid, we can wait
                    let barrier_data = others::barrier::as_ref();
                    if barrier_data.is_world_valid() {
                        if barrier_data.is_world_destroyed() {
                            println!("Stopping the render thread...");
                            barrier_data.thread_sync_quit();
                            break;
                        }
                    }             
                }
                println!("Stopped the render thread!");
            })
            .unwrap();
    };
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    barrier.wait();
    println!("Successfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());
    PipelineStartData { handle: join_handle, rx: rx2 }
}

// Commands that can be ran internally
pub fn internal_task(buf: &mut PipelineBuffer, task: RenderTask) -> (Option<GPUObjectID>, Option<AsyncGPUCommandData>) {
    // Handle the internal case
    match task {
        // Shaders
        RenderTask::SubShaderCreate(shared_shader) => (Some(object_creation::create_compile_subshader(buf, shared_shader)), None),
        RenderTask::ShaderCreate(shared_shader) => (Some(object_creation::create_compile_shader(buf, shared_shader)), None),
        // Textures
        RenderTask::TextureCreate(shared_texture) => (Some(object_creation::generate_texture(buf, shared_texture)), None),
        RenderTask::TextureUpdateSize(id, ttype) => {
            object_creation::update_texture_size(buf, id, ttype);
            (None, None)
        }
        RenderTask::TextureUpdateData(id, bytes) => {
            object_creation::update_texture_data(buf, id, bytes);
            (None, None)
        }
        RenderTask::TextureFillArray(id, bytecount_per_pixel, return_bytes) =>  {
            object_creation::texture_fill_array(buf, id, bytecount_per_pixel, return_bytes);
            (None, None)
        }
        // Model
        RenderTask::ModelCreate(shared_model) => (Some(object_creation::create_model(buf, shared_model)), None),
        RenderTask::ModelDispose(gpumodel) => {
            object_creation::dispose_model(buf, gpumodel);
            (None, None)
        }
        // Material
        RenderTask::MaterialCreate(material) => (Some(object_creation::create_material(buf, material)), None),
        // Compute
        RenderTask::ComputeRun(id, axii, uniforms_group) => (None, Some(object_creation::run_compute(buf, id, axii, uniforms_group))),
        // Others
        _ => (None, None),
    }
}

// Run a command on the Render Thread
fn command(lock: &mut CommandExecutionResults, buf: &mut PipelineBuffer, renderer: &mut PipelineRenderer, camera: &mut CameraDataGPUObject, command: RenderCommandQuery, _window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
    // Handle the common cases
    let (command_result, async_command_data) = match command.task {
        // Window tasks
        RenderTask::WindowUpdateFullscreen(fullscreen) => {
            renderer.window.fullscreen = fullscreen;
            (None, None)
        }
        RenderTask::WindowUpdateVSync(vsync) => {
            renderer.window.vsync = vsync;
            // We need an OpenGL context to do this shit
            if vsync {
                // Enable VSync
                glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            } else {
                // Disable VSync
                glfw.set_swap_interval(glfw::SwapInterval::None);
            }
            (None, None)
        }
        RenderTask::WindowUpdateSize(size) => {
            renderer.window.dimensions = size;
            renderer.update_window_dimensions(size);
            (None, None)
        }
        // Pipeline
        RenderTask::CameraDataUpdate(shared) => {
            let (pos, rot, clip_planes, projm) = shared.get();
            // Calculate the view matrix using the position and rotation
            let viewm = {
                let rm = veclib::Matrix4x4::from_quaternion(&rot);
                let forward_vector = rm.mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, -1.0)).normalized();
                let up_vector = rm.mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0)).normalized();
                veclib::Matrix4x4::look_at(&pos, &up_vector, &(forward_vector + pos))
            };
            *camera = CameraDataGPUObject {
                position: pos,
                rotation: rot,
                clip_planes,
                viewm,
                projm,
            };
            (None, None)
        }
        // Renderer commands
        RenderTask::RendererAdd(shared_renderer) => (Some(object_creation::add_renderer(buf, shared_renderer)), None),
        RenderTask::RendererRemove(renderer_id) => {
            object_creation::remove_renderer(buf, &renderer_id);
            (None, None)
        }
        RenderTask::RendererUpdateTransform(renderer_id, matrix) => {
            object_creation::update_renderer(buf, &renderer_id, matrix);
            (None, None)
        }
        // Internal cases
        x => internal_task(buf, x),
    };
    // Extract
    let command_id = command.command_id;
    let callback_id = command.callback_id.clone();
    let thread_id = command.thread_id;
    let callback_id = match callback_id {
        Some(x) => Some((x, thread_id)),
        None => None,
    };
    // If this is an async GPU task (Like running a compute shader) we will not call the callbacks
    if let Option::Some(mut x) = async_command_data {
        // We must buffer the async command data, so we can poll it and check if it was executed later
        x.additional_command_data((command_id, callback_id));
        buf.add_async_gpu_command_data(x);
    } else {
        // This is not an async GPU task, we can buffer the callbacks directly
        buf.received_new_gpuobject_additional(command_result.clone(), callback_id);
        crate::others::executed_command(command_id, command_result, lock);    
    }
}

// Poll commands that have been sent to us by the worker threads OR the main thread
fn poll_commands(buf: &mut PipelineBuffer, renderer: &mut PipelineRenderer, camera: &mut CameraDataGPUObject, rx: &Receiver<RenderCommandQuery>, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
    // We must loop through every command that we receive from the main thread
    let mut i = 0;    
    let mut lock = RESULT.lock().unwrap();
    let lock = &mut *lock;
    for render_command_query in rx.try_iter() {
        i+=1;        
        // Check special commands first
        // Valid command
        command(lock, buf, renderer, camera, render_command_query, window, glfw);
    }
    //println!("Executed {} Render Commands", i);
}

// Check if any of the async GPU commands have finished executing
fn poll_async_gpu_commands(buf: &mut PipelineBuffer) {
    let mut datas: Vec<(u64, Option<(u64, std::thread::ThreadId)>)> = Vec::new();
    buf.async_gpu_command_datas.retain(|async_gpu_command_data| {
        // Check if the OpenGL command was executed
        if async_gpu_command_data.has_executed() {
            // The OpenGL command did executed, so we must tell signal the threads to run their callbacks
            let x = async_gpu_command_data.command_data.unwrap();
            datas.push(x);             
            false
        } else { true }
    });
    // Inform the other thread
    let mut lock = RESULT.lock().unwrap();
    for (command_id, callback_id) in datas {
        buf.received_new_gpuobject_additional(None, callback_id);
        crate::others::executed_command(command_id, None, &mut *lock);   
    }
}
// Data that will be sent back to the main thread after we start the pipeline thread
pub struct PipelineStartData {
    pub handle: std::thread::JoinHandle<()>,
    pub rx: std::sync::mpsc::Receiver<MainThreadMessage>,
}

mod object_creation {
    use std::{ffi::{CString, c_void}, ptr::null, mem::size_of, sync::{atomic::{AtomicPtr, Ordering}, Mutex, Arc}};

    use crate::{Renderer, SharedData, GPUObjectID, RendererGPUObject, GPUObject, pipeline::{buffer::PipelineBuffer, async_command_data::AsyncGPUCommandData}, SubShader, SubShaderType, SubShaderGPUObject, Shader, ComputeShaderGPUObject, ShaderGPUObject, Model, ModelGPUObject, Texture, TextureType, TextureFilter, TextureFlags, TextureWrapping, TextureGPUObject, ShaderUniformsGroup, Material, MaterialGPUObject};
    // Add the renderer to the renderer (lol I need better name)
    pub fn add_renderer(buf: &mut PipelineBuffer, renderer: SharedData<(Renderer, veclib::Matrix4x4<f32>)>) -> GPUObjectID {
        let (renderer, matrix) = renderer.get();
        let material_id = renderer.material.unwrap();
        let model_id = renderer.model.clone().unwrap();
        let renderer_gpuobject = GPUObject::Renderer(RendererGPUObject {
            model_id,
            material_id,
            matrix,
        });
        let id = buf.add_gpuobject(renderer_gpuobject, None);
        buf.renderers.insert(id.clone());
        id
    }
    // Remove the renderer using it's renderer ID
    pub fn remove_renderer(buf: &mut PipelineBuffer, renderer_id: &GPUObjectID) {
        // Remove first
        buf.remove_gpuobject(renderer_id.clone());
        buf.renderers.remove(renderer_id);
    }
    // Update a renderer's model matrix
    pub fn update_renderer(buf: &mut PipelineBuffer, renderer_id: &GPUObjectID, matrix: SharedData<veclib::Matrix4x4<f32>>) {
        let renderer = buf.as_renderer_mut(renderer_id).unwrap();
        renderer.matrix = matrix.get();
    }
    pub fn create_compile_subshader(buf: &mut PipelineBuffer, subshader: SharedData<SubShader>) -> GPUObjectID {
        let shader_type: u32;
        let subshader = subshader.get();
        match subshader.subshader_type {
            SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
            SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
            SubShaderType::Compute => shader_type = gl::COMPUTE_SHADER,
        }
        unsafe {
            let program = gl::CreateShader(shader_type);
            // Compile the shader
            let cstring = CString::new(subshader.source.clone()).unwrap();
            let shader_source: *const i8 = cstring.as_ptr();
            gl::ShaderSource(program, 1, &shader_source, null());
            gl::CompileShader(program);
            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            // Print any errors that might've happened while compiling this subshader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetShaderInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while compiling sub-shader {}!:", subshader.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();

                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                println!("{}", subshader.source);
                panic!();
            }

            println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", subshader.name);
            // Add the gpu object
            let gpuobject = GPUObject::SubShader(SubShaderGPUObject {
                subshader_type: subshader.subshader_type,
                program,
            });
            buf.add_gpuobject(gpuobject, Some(subshader.name.clone()))
        }
    }
    pub fn create_compile_shader(buf: &mut PipelineBuffer, shader: SharedData<Shader>) -> GPUObjectID {
        let shader = shader.get();
        println!("\x1b[33mCompiling & Creating Shader {}...\x1b[0m", shader.name);
        unsafe {
            let program = gl::CreateProgram();

            // Attach the shader
            let mut subshaders: Vec<&SubShaderGPUObject> = Vec::new();
            for subshader_id in shader.linked_subshaders.iter() {
                let subshader = buf.as_subshader(subshader_id).unwrap();
                gl::AttachShader(program, subshader.program);
                subshaders.push(subshader);
            }

            // Finalize the shader and stuff
            gl::LinkProgram(program);

            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetProgramiv(program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while finalizing this shader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetProgramInfoLog(program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while finalizing shader {}!:", shader.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }
            // Check if this a compute shader
            let compute_shader = if let SubShaderType::Compute = subshaders.first().unwrap().subshader_type { true } else { false };
            // Detach shaders
            for subshader in subshaders {
                gl::DetachShader(program, subshader.program);
            }
            let gpuobject = if !compute_shader {
                // Normal shader
                GPUObject::Shader(ShaderGPUObject {
                    program,
                })
            } else {
                // Compute shader
                GPUObject::ComputeShader(ComputeShaderGPUObject {
                    program,
                })
            };
            // Add the gpu object
            println!("\x1b[32mShader {} compiled and created succsessfully! ComputeShader: {}\x1b[0m", shader.name, compute_shader);
            buf.add_gpuobject(gpuobject, Some(shader.name.clone()))
        }
    }
    pub fn create_model(buf: &mut PipelineBuffer, model: SharedData<Model>) -> GPUObjectID {
        let model = model.get();
        let mut gpu_data = ModelGPUObject {
            vertex_buf: 0,
            normal_buf: 0,
            uv_buf: 0,
            tangent_buf: 0,
            color_buf: 0,
            vertex_array_object: 0,
            element_buffer_object: 0,
            element_count: 0,
        };
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut gpu_data.vertex_array_object);
            gl::BindVertexArray(gpu_data.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut gpu_data.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gpu_data.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (model.triangles.len() * size_of::<u32>()) as isize,
                model.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut gpu_data.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.vertices.len() * size_of::<f32>() * 3) as isize,
                model.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut gpu_data.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (model.normals.len() * size_of::<f32>() * 3) as isize,
                model.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            if !model.tangents.is_empty() {
                // And it's brother, the tangent buffer
                gl::GenBuffers(1, &mut gpu_data.tangent_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.tangents.len() * size_of::<f32>() * 4) as isize,
                    model.tangents.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.uvs.is_empty() {
                // The texture coordinates buffer
                gl::GenBuffers(1, &mut gpu_data.uv_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.uvs.len() * size_of::<f32>() * 2) as isize,
                    model.uvs.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !model.colors.is_empty() {
                // Finally, the vertex colors buffer
                gl::GenBuffers(1, &mut gpu_data.color_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (model.colors.len() * size_of::<f32>() * 3) as isize,
                    model.colors.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }
            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            if !model.tangents.is_empty() {
                // Tangent attribute
                gl::EnableVertexAttribArray(2);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.uvs.is_empty() {
                // UV attribute
                gl::EnableVertexAttribArray(3);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
                gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !model.colors.is_empty() {
                // Vertex color attribute
                gl::EnableVertexAttribArray(4);
                gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
            }
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        gpu_data.element_count = model.triangles.len();
        // Add the gpu object
        let gpuobject = GPUObject::Model(gpu_data);
        buf.add_gpuobject(gpuobject, Some(model.name))
    }
    pub fn dispose_model(buf: &mut PipelineBuffer, id: GPUObjectID) {
        // Get the model GPU object first
        let gpuobject = buf.get_gpuobject_mut(&id).unwrap();
        let model = if let GPUObject::Model(x) = gpuobject {
            x
        } else { panic!(); };
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(1, &mut model.vertex_buf);
            gl::DeleteBuffers(1, &mut model.normal_buf);
            gl::DeleteBuffers(1, &mut model.uv_buf);
            gl::DeleteBuffers(1, &mut model.tangent_buf);
            gl::DeleteBuffers(1, &mut model.color_buf);
            gl::DeleteBuffers(1, &mut model.element_buffer_object);

            // Delete the vertex array
            gl::DeleteVertexArrays(1, &mut model.vertex_array_object);
        }
    }
    pub fn generate_texture(buf: &mut PipelineBuffer, texture: SharedData<Texture>) -> GPUObjectID {
        let mut pointer: *const c_void = null();
        let texture = texture.get();
        if !texture.bytes.is_empty() {
            pointer = texture.bytes.as_ptr() as *const c_void;
        }
        let ifd = crate::get_ifd(texture._format, texture._type);

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match texture.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // It's a normal mutable texture
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id as *mut u32);
            gl::BindTexture(tex_type, id);
            match texture.ttype {
                TextureType::Texture1D(width) => {
                    gl::TexImage1D(tex_type, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a 2D texture
                TextureType::Texture2D(width, height) => {
                    gl::TexImage2D(tex_type, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a 3D texture
                TextureType::Texture3D(width, height, depth) => {
                    gl::TexImage3D(tex_type, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                }
                // This is a texture array
                TextureType::TextureArray(width, height, depth) => {
                    gl::TexStorage3D(
                        tex_type,
                        Texture::guess_mipmap_levels(width.max(height) as usize) as i32,
                        ifd.0 as u32,
                        width as i32,
                        height as i32,
                        depth as i32,
                    );
                    // We might want to do mipmap
                    for i in 0..depth {
                        let localized_bytes = texture.bytes[(i as usize * height as usize * 4 * width as usize)..texture.bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, ifd.1, ifd.2, localized_bytes);
                    }
                }
            }
            // Set the texture parameters for a normal texture
            match texture.filter {
                TextureFilter::Linear => {
                    // 'Linear' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }
            }
        }

        // The texture is already bound to the TEXTURE_2D
        if texture.flags.contains(TextureFlags::MIPMAPS) {
            // Create the mipmaps
            unsafe {
                gl::GenerateMipmap(tex_type);
                // Set the texture parameters for a mipmapped texture
                match texture.filter {
                    TextureFilter::Linear => {
                        // 'Linear' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    }
                }
            }
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode: i32;
        match texture.wrap_mode {
            TextureWrapping::ClampToEdge => wrapping_mode = gl::CLAMP_TO_EDGE as i32,
            TextureWrapping::ClampToBorder => wrapping_mode = gl::CLAMP_TO_BORDER as i32,
            TextureWrapping::Repeat => wrapping_mode = gl::REPEAT as i32,
            TextureWrapping::MirroredRepeat => wrapping_mode = gl::MIRRORED_REPEAT as i32,
        }
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode);
        }
        println!("RenderThread: Succsesfully generated texture {}", texture.name);
        let gpuobject = GPUObject::Texture(TextureGPUObject {
            texture_id: id,
            ifd,
            ttype: texture.ttype,
        });
        buf.add_gpuobject(gpuobject, Some(texture.name.clone()))
    }
    pub fn update_texture_size(buf: &mut PipelineBuffer, id: GPUObjectID, ttype: TextureType) {
        // Get the GPU texture object
        let texture = if let GPUObject::Texture(x) = buf.get_gpuobject(&id).unwrap() {
            x
        } else { panic!() };
        // Check if the current dimension type matches up with the new one
        let ifd = texture.ifd;
        // This is a normal texture getting resized
        unsafe {
            match ttype {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, texture.texture_id);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, texture.texture_id);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, texture.texture_id);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
    }
    pub fn update_texture_data(buf: &mut PipelineBuffer, id: GPUObjectID, bytes: Vec<u8>) {
        let texture = if let GPUObject::Texture(x) = buf.get_gpuobject(&id).unwrap() {
            x
        } else { panic!() };
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        let (internal_format, format, data_type) = texture.ifd;
        let tex_type = match texture.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        unsafe {
            gl::BindTexture(tex_type, texture.texture_id);
            match texture.ttype {
                TextureType::Texture1D(width) => gl::TexImage1D(tex_type, 0, internal_format, width as i32, 0, format, data_type, pointer),
                // This is a 2D texture
                TextureType::Texture2D(width, height) => {
                    gl::TexImage2D(tex_type, 0, internal_format, width as i32, height as i32, 0, format, data_type, pointer);
                }
                // This is a 3D texture
                TextureType::Texture3D(width, height, depth) => {
                    gl::TexImage3D(tex_type, 0, internal_format, width as i32, height as i32, depth as i32, 0, format, data_type, pointer);
                }
                // This is a texture array
                TextureType::TextureArray(width, height, depth) => {
                    gl::TexStorage3D(tex_type, 10, internal_format as u32, width as i32, height as i32, depth as i32);
                    // We might want to do mipmap
                    for i in 0..depth {
                        let localized_bytes = bytes[(i as usize * height as usize * 4 * width as usize)..bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, format, data_type, localized_bytes);
                    }
                }
            }
        }
    }
    pub fn texture_fill_array(buf: &mut PipelineBuffer, id: GPUObjectID, bytecount_per_pixel: usize, return_bytes: Arc<AtomicPtr<Vec<u8>>>) {
        let texture = if let GPUObject::Texture(x) = buf.get_gpuobject(&id).unwrap() {
            x
        } else { panic!() };
        // Get the length of the vector
        let length: usize = match texture.ttype {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Get the byte size
        let byte_length = bytecount_per_pixel * length;

        // Create the vector
        let mut pixels: Vec<u8> = vec![0; byte_length];

        let tex_type = match texture.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, texture.texture_id);
            let (_internal_format, format, data_type) = texture.ifd;
            gl::GetTexImage(tex_type, 0, format, data_type, pixels.as_mut_ptr() as *mut c_void);
        }

        // Update the vector that was given using the AtomicPtr
        let new_bytes = return_bytes.as_ref().load(Ordering::Relaxed); 
        unsafe { 
            *new_bytes = pixels;
        }
    }
    pub fn run_compute(buf: &mut PipelineBuffer, id: GPUObjectID, axii: (u16, u16, u16), uniforms_group: ShaderUniformsGroup) -> AsyncGPUCommandData {
        unsafe { gl::Flush(); }
        // Dispatch the compute shader for execution
        uniforms_group.consume(buf).unwrap();
        let sync = unsafe {
            gl::DispatchCompute(axii.0 as u32, axii.1 as u32, axii.2 as u32);
            gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)
        };
        fn callback(id: GPUObjectID, buf: &mut PipelineBuffer) {
            
        }
        AsyncGPUCommandData::new(sync, Some(id), callback)
    }
    pub fn create_material(buf: &mut PipelineBuffer, material: SharedData<Material>) -> GPUObjectID {
        let material = material.get();
        let gpuobject = GPUObject::Material(MaterialGPUObject {
            shader: material.shader,
            uniforms: material.uniforms,
            flags: material.flags,
        });
        buf.add_gpuobject(gpuobject, Some(material.material_name.clone()))
    }
}
