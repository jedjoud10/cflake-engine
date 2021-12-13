use glfw::Context;
use super::object::*;
use lazy_static::lazy_static;
use crate::{basics::*, pipec, rendering::PipelineRenderer, RenderCommand, RenderTask, RenderTaskReturn, SharedData};
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
    sync::{
        atomic::AtomicPtr,
        mpsc::{Receiver, Sender},
        Arc, Mutex, RwLock, MutexGuard, Barrier,
    },
};

lazy_static! {
    // The pipeline that is stored on the render thread
    pub static ref PIPELINE: Arc<no_deadlocks::RwLock<Option<Pipeline>>> = Arc::new(no_deadlocks::RwLock::new(None));
}

// The data that will be sent to the pipeline
pub struct PipelineSendData(pub std::thread::ThreadId, pub RenderCommand, pub Box<dyn Fn(GPUObject) + Send + Sync + 'static>, pub bool);

// Get an immutable lock of the render pipeline
pub fn pipeline() -> no_deadlocks::RwLockReadGuard<'static, Option<Pipeline>> {
    let x = PIPELINE.read().unwrap();
    x
}

pub fn pipeline_mut() -> no_deadlocks::RwLockWriteGuard<'static, Option<Pipeline>> {
    let x = PIPELINE.write().unwrap();
    x
}

// Create the new render thread
pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
    println!("Initializing RenderPipeline...");
    // Create a single channel (WorkerThreads/MainThread  => Render Thread)
    let (tx, rx): (Sender<PipelineSendData>, Receiver<PipelineSendData>) = std::sync::mpsc::channel(); // Main to render
    // Barrier so we can wait until the render thread has finished initializing
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();
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
        builder.spawn(move || {
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
            let threads_to_render = rx;

            // Set the pipeline
            let pipeline = Pipeline {
                gpu_objects: HashMap::new(),
                tx_template: tx,
            };
            {
                let mut p = crate::pipeline::pipeline_mut();
                *p = Some(pipeline);
            }
            // This is indeed the render thread
            crate::pipeline::IS_RENDER_THREAD.with(|x| x.set(true));
            // Initialize the deferred renderer
            let mut pipeline_renderer = PipelineRenderer::default();
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
            // If the render pipeline and thread are valid
            let mut valid = true;
            println!("Successfully created the RenderThread!");
            barrier_clone.wait();
            while valid {
                // Update the delta_time
                let new_time = glfw.get_time();
                let delta = new_time - last_time;
                last_time = new_time;
                // Run the frame
                frame(
                    glfw.borrow_mut(),
                    window.borrow_mut(),
                    &threads_to_render,
                    &mut pipeline_renderer,
                    &mut camera,
                    &mut valid,
                    frame_count,
                    delta,
                );
                frame_count += 1;
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            println!("Stopping the render thread!");
        }).unwrap();
    };
    // Wait for the init message...
    let i = std::time::Instant::now();
    println!("Waiting for RenderThread init confirmation...");
    barrier.wait();
    println!("RSuccessfully initialized the RenderPipeline! Took {}ms to init RenderThread", i.elapsed().as_millis());
}

// Commands that can be ran internally
pub fn internal_task(task: RenderTask) -> GPUObject {
    // Handle the internal case
    match task {
        // Shaders
        RenderTask::SubShaderCreate(shared_shader) => Pipeline::create_compile_subshader(shared_shader),
        RenderTask::ShaderCreate(shared_shader) => Pipeline::create_compile_shader(shared_shader),
        RenderTask::ShaderUniformGroup(_shared_uniformgroup) => todo!(),
        // Textures
        RenderTask::TextureCreate(shared_texture) => Pipeline::generate_texture(shared_texture),
        RenderTask::TextureUpdateSize(texture, ttype) => {
            Pipeline::update_texture_size(texture, ttype);
            GPUObject::None
        }
        RenderTask::TextureUpdateData(texture, bytes) => {
            Pipeline::update_texture_data(texture, bytes);
            GPUObject::None
        }
        RenderTask::TextureFillArray(texture, bytecount) => Pipeline::texture_fill_array(texture, bytecount),
        // Model
        RenderTask::ModelCreate(shared_model) => Pipeline::create_model(shared_model),
        RenderTask::ModelDispose(gpumodel) => {
            Pipeline::dispose_model(gpumodel);
            GPUObject::None
        }
        // Compute
        RenderTask::ComputeRun(compute, axii, uniforms_group) => {
            Pipeline::run_compute(compute, axii, uniforms_group);
            GPUObject::None
        }
        RenderTask::ComputeLock(compute) => {
            Pipeline::lock_compute(compute);
            GPUObject::None
        }
        // Others
        _ => GPUObject::None,
    }
}

// Run a command on the Render Thread
fn command(
    name: String,
    pr: &mut PipelineRenderer,
    camera: &mut CameraDataGPUObject,
    command: RenderCommand,
    window: &mut glfw::Window,
    glfw: &mut glfw::Glfw,
) -> RenderTaskReturn {
    // Handle the common cases
    match command.input_task {
        // Window tasks
        RenderTask::WindowUpdateFullscreen(fullscreen) => {
            pr.window.fullscreen = fullscreen;
            RenderTaskReturn::None
        }
        RenderTask::WindowUpdateVSync(vsync) => {
            pr.window.vsync = vsync;
            // We need an OpenGL context to do this shit
            if vsync {
                // Enable VSync
                glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            } else {
                // Disable VSync
                glfw.set_swap_interval(glfw::SwapInterval::None);
            }
            RenderTaskReturn::None
        }
        RenderTask::WindowUpdateSize(size) => {
            pr.window.dimensions = size;
            pr.update_window_dimensions(size);
            RenderTaskReturn::None
        }
        // Pipeline
        RenderTask::DestroyRenderThread => todo!(),
        RenderTask::CameraDataUpdate(shared) => {
            let pos = shared.object.0;
            let rot = shared.object.1;
            let clip_planes = shared.object.2;
            let projm = shared.object.3;
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
            RenderTaskReturn::None
        }
        // Renderer commands
        RenderTask::RendererAdd(shared_renderer) => {
            RenderTaskReturn::GPUObject(Pipeline::add_renderer(pr, shared_renderer), name)
        }
        RenderTask::RendererRemove(renderer_id) => {
            Pipeline::remove_renderer(pr, renderer_id);
            RenderTaskReturn::None
        }
        RenderTask::RendererUpdateTransform(_renderer, _transform) => todo!(),
        // Internal cases
        x => RenderTaskReturn::GPUObject(internal_task(x), name),
    }
}

// Poll commands that have been sent to us by the main thread
fn poll_commands(
    pr: &mut PipelineRenderer,
    camera: &mut CameraDataGPUObject,
    rx: &Receiver<PipelineSendData>,
    valid: &mut bool,
    window: &mut glfw::Window,
    glfw: &mut glfw::Glfw,
) {
    // We must loop through every command that we receive from the main thread
    for pipeline_data in rx.try_iter() {
        let (thread_id, cmd, callback, waitable) = (pipeline_data.0, pipeline_data.1, pipeline_data.2, pipeline_data.3);
        // Check special commands first
        let name = cmd.name.clone();
        match cmd.input_task {
            // Pipeline shit
            RenderTask::DestroyRenderThread => {
                // Destroy the render thread
                *valid = false;
                break;
            }
            _ => {
                // Valid command
                match command(name.clone(), pr, camera, cmd, window, glfw) {
                    RenderTaskReturn::None => { /* Not valid */ }
                    taskreturn => {
                        match taskreturn {
                            RenderTaskReturn::None => { /* Not valid */ } ,
                            RenderTaskReturn::GPUObject(object, name) => /* We *might* have a GPU object */ {
                                match object {
                                    GPUObject::None => {
                                        /* Not valid */
                                    },
                                    gpuobject => {
                                        // We have a valid GPU object
                                        // We must somehow find a way to send this to the Interface
                                        super::interface::executed_task(thread_id, name, gpuobject, callback, waitable)
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// The render thread that is continuously being ran
fn frame(
    _glfw: &mut glfw::Glfw,
    window: &mut glfw::Window,
    rx: &Receiver<PipelineSendData>,
    pipeline_renderer: &mut PipelineRenderer,
    camera: &mut CameraDataGPUObject,
    valid: &mut bool,
    _frame_count: u128,
    _delta_time: f64,
) {
    // Poll first
    poll_commands(pipeline_renderer, camera, rx, valid, window, _glfw);
    // Pre-render
    pipeline_renderer.pre_render();
    // Render
    pipeline_renderer.renderer_frame(camera);
    // Post-render
    pipeline_renderer.post_render(camera, window);
    // Remove the already called callbacks
    super::interface::update_render_thread();
}

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
pub struct Pipeline {
    pub gpu_objects: HashMap<String, GPUObject>,                                   // The GPU objects that where generated on the Rendering Thread and sent back to the main thread
    pub tx_template: Sender<PipelineSendData> // A copy of the sender so we can copy it on each thread and make it thread local
}
impl Pipeline {    
    // Get GPU object using it's specified name
    pub fn get_gpu_object(&self, name: &str) -> Option<&GPUObject> {
        self.gpu_objects.get(name)
    }
    // Check if a GPU object exists
    pub fn gpu_object_valid(&self, name: &str) -> bool {
        self.gpu_objects.contains_key(name)
    }
}
// Renderers
impl Pipeline {
    // Add the renderer to the renderer (lol I need better name)
    pub fn add_renderer(pr: &mut PipelineRenderer, renderer: SharedData<(Renderer, veclib::Matrix4x4<f32>)>) -> GPUObject {
        let matrix = &renderer.object.1;
        let renderer = &renderer.object.0;
        let model = renderer.model.clone();
        let material = Self::create_material(renderer.material.clone(), pr.default_material.as_ref().unwrap());
        let renderer_gpuobject = RendererGPUObject(model, material, *matrix);
        let x = pr.add_renderer(renderer_gpuobject);
        GPUObject::Renderer(x)
    }
    // Remove the renderer using it's renderer ID
    pub fn remove_renderer(pr: &mut PipelineRenderer, renderer_id: usize) {
        pr.remove_renderer(renderer_id);
    }
}

// The actual OpenGL tasks that are run on the render thread
impl Pipeline {
    pub fn create_compile_subshader(subshader: SharedData<SubShader>) -> GPUObject {
        let shader_type: u32;
        let subshader = subshader.object.as_ref();
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
            GPUObject::SubShader(SubShaderGPUObject(subshader.subshader_type, program))
        }
    }
    pub fn create_compile_shader(shader: SharedData<Shader>) -> GPUObject {
        let shader = shader.object.as_ref();
        unsafe {
            let program = gl::CreateProgram();

            // Attach the shaders
            for subshader_program in shader.linked_subshaders_programs.iter() {
                gl::AttachShader(program, subshader_program.1);
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
            let compute_shader: bool = match shader.linked_subshaders_programs.get(0).unwrap().0 {
                SubShaderType::Compute => true,
                _ => false,
            };
            // Detach shaders
            for subshader_program in shader.linked_subshaders_programs.iter() {
                gl::DetachShader(program, subshader_program.1);
            }
            if !compute_shader {
                // Normal shader
                GPUObject::Shader(ShaderGPUObject(program))
            } else {
                // Compute shader
                GPUObject::ComputeShader(ComputeShaderGPUObject(program))
            }
        }
    }
    pub fn create_model(model: SharedData<Model>) -> GPUObject {
        let model = model.object.as_ref();
        let mut gpu_data = ModelGPUObject::default();
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
        GPUObject::Model(gpu_data)
    }
    pub fn dispose_model(mut model: ModelGPUObject) {
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
    pub fn generate_texture(texture: SharedData<Texture>) -> GPUObject {
        let mut pointer: *const c_void = null();
        let texture = texture.object.as_ref();
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
        GPUObject::Texture(TextureGPUObject(id, ifd, texture.ttype))
    }
    pub fn update_texture_size(texture: TextureGPUObject, ttype: TextureType) {
        // Check if the current dimension type matches up with the new one
        let ifd = texture.1;
        // This is a normal texture getting resized
        unsafe {
            match ttype {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, texture.0);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, texture.0);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, texture.0);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, null());
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
    }
    pub fn update_texture_data(texture: TextureGPUObject, bytes: Vec<u8>) {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        let (internal_format, format, data_type) = texture.1;
        let tex_type = match texture.2 {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        unsafe {
            gl::BindTexture(tex_type, texture.0);
            match texture.2 {
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
    pub fn texture_fill_array(texture: TextureGPUObject, bytecount: usize) -> GPUObject {
        // Get the length of the vector
        let length: usize = match texture.2 {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Get the byte size
        let byte_length = bytecount * length;

        // Create the vector
        let mut pixels: Vec<u8> = vec![0; byte_length];

        let tex_type = match texture.2 {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, texture.0);
            let (_internal_format, format, data_type) = texture.1;
            gl::GetTexImage(tex_type, 0, format, data_type, pixels.as_mut_ptr() as *mut c_void);
        }
        GPUObject::TextureFill(TextureFillGPUObject(pixels, bytecount))
    }
    pub fn run_compute(compute: ComputeShaderGPUObject, axii: (u16, u16, u16), uniforms_group: ShaderUniformsGroup) {
        uniforms_group.consume();
        unsafe {
            gl::DispatchCompute(axii.0 as u32, axii.1 as u32, axii.2 as u32);
        }
    }
    pub fn lock_compute(compute: ComputeShaderGPUObject) {
        unsafe {
            // Remember to use the shader first
            gl::UseProgram(compute.0);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }
    }
    pub fn create_material(material: Material, default_material: &Material) -> MaterialGPUObject {
        let default_shader = default_material.shader.as_ref().unwrap().clone();
        MaterialGPUObject(material.shader.unwrap_or(default_shader), material.uniforms, material.flags)
    }
}
