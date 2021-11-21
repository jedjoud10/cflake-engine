use std::ffi::CString;
use std::ptr::null;

use crate::{DataType, Uniform};
use crate::{FrameStats, Material, MaterialFlags, Renderer, Texture, TextureType, pipec, pipeline::object::*};
use crate::basics::texture::*;
// The main renderer, this is stored
#[derive(Default)]
pub struct PipelineRenderer {
    pub framebuffer: u32,
    // The frame buffer textures
    pub diffuse_texture: TextureGPUObject,
    pub normals_texture: TextureGPUObject,
    pub position_texture: TextureGPUObject,
    pub depth_texture: TextureGPUObject,
    pub debug_view: u16,
    pub wireframe: bool,
    wireframe_shader: ShaderGPUObject,
    frame_stats: FrameStats,
    quad_renderer: Renderer,
}

// Setup all the settings for OpenGL like culling and the clear color
pub fn init_deferred_renderer(renderer: &mut PipelineRenderer, dimensions: veclib::Vector2<u16>) {
    // Local function for binding a texture to a specific frame buffer attachement
    fn bind_attachement(attachement: u32, texture: &TextureGPUObject) {
        unsafe {
            // Default target, no multisamplind
            let target: u32 = gl::TEXTURE_2D;
            gl::BindTexture(target, texture.0);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.0, 0);
        }
    }  
    unsafe {
        gl::GenFramebuffers(1, &mut renderer.framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, renderer.framebuffer);
        let dims = TextureType::Texture2D(dimensions.x, dimensions.y);
        // Create the diffuse render texture
        renderer.diffuse_texture = pipec::texture(Texture::default()
            .set_dimensions(dims)
            .set_format(TextureFormat::RGB32F));
        // Create the normals render texture
        renderer.normals_texture = pipec::texture(Texture::default()
            .set_dimensions(dims)
            .set_format(TextureFormat::RGB8RS));
        // Create the position render texture
        renderer.position_texture = pipec::texture(Texture::default()
            .set_dimensions(dims)
            .set_format(TextureFormat::RGB32F));
        // Create the depth render texture
        renderer.depth_texture = pipec::texture(Texture::default()
            .set_dimensions(dims)
            .set_format(TextureFormat::DepthComponent32)
            .set_data_type(DataType::Float32));
        
        // Now bind the attachememnts
        bind_attachement(gl::COLOR_ATTACHMENT0, &renderer.diffuse_texture);
        bind_attachement(gl::COLOR_ATTACHMENT1, &renderer.normals_texture);
        bind_attachement(gl::COLOR_ATTACHMENT2, &renderer.position_texture);
        bind_attachement(gl::DEPTH_ATTACHMENT, &renderer.depth_texture);
        let attachements = vec![gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
        gl::DrawBuffers(attachements.len() as i32, attachements.as_ptr() as *const u32);

        // Check if the frame buffer is okay
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
        } else {
            panic!("Framebuffer has failed initialization! Error: '{}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
        }

        // Unbind
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}

// Render a renderer normally
pub fn render(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject) {
    let shader = (renderer.1).0;
    let material = renderer.1;
    let model = renderer.0;
    let mut group = shader.new_uniform_group();
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    // Pass the MVP and the model matrix to the shader
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    group.set_vec3f32("view_pos", camera.position);
    group.consume();

    unsafe {
        // Enable / Disable vertex culling for double sided materials
        if material.2.contains(MaterialFlags::DOUBLE_SIDED) { gl::Disable(gl::CULL_FACE); }            
        else { gl::Enable(gl::CULL_FACE); }

        // Actually draw
        gl::BindVertexArray(model.0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.1);
        gl::DrawElements(gl::TRIANGLES, model.2 as i32, gl::UNSIGNED_INT, null());
    }
}

// Render a multi-material renderer
fn render_mm(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject) {
    // TODO: Gotta reprogram the multi material system now
}

// Render a renderer using wireframe 
fn render_wireframe(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject, ws: &ShaderGPUObject) {
    let shader = (renderer.1).0;
    let material = renderer.1;
    let model = renderer.0;
    let mut group = ws.new_uniform_group();
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    unsafe {
        // Set the wireframe rendering
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::Enable(gl::LINE_SMOOTH);

        gl::BindVertexArray(model.0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.1);
        gl::DrawElements(gl::TRIANGLES, model.2 as i32, gl::UNSIGNED_INT, null());

        // Reset the wireframe settings
        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::Disable(gl::LINE_SMOOTH);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
    }
}

// Events
fn system_enabled(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();

    // Create the screen quad
    system.create_screen_quad(data);

    // Load volumetric stuff
    system.volumetric.load_compute_shaders(&mut data.asset_manager);
    system.volumetric.create_textures(data.custom_data.window.dimensions, 64, 4);
    system.volumetric.generate_sdf(&mut data.asset_manager);
    system.volumetric.disable();

    // Get the OpenGL version
    unsafe {
        let mut major: i32 = 0;
        let mut minor: i32 = 0;
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        // Error shit
    }

    // Then setup opengl and the render buffer
    system.setup_opengl(data);
    let material = &system.quad_renderer.material;
    let shader = material.shader.as_ref().unwrap();

    // Set the default uniforms
    /*
    /*
    // Volumetric parameters
    shader.set_t2d("volumetric_texture", &system.volumetric.result_tex, gl::TEXTURE6);
    shader.set_t2d("volumetric_depth_texture", &system.volumetric.depth_tex, gl::TEXTURE7);
    shader.set_t3d("sdf_texture", &system.volumetric.sdf_tex, gl::TEXTURE8);
    */
    */

    // Load the compute shader for the frame stats
    system.frame_stats.load_compute_shader(data.asset_manager);

    // Load sky gradient texture
    let texture = Texture::default()
        .set_wrapping_mode(TextureWrapping::ClampToEdge)
        .cache_load("defaults\\textures\\sky_gradient.png", data.asset_manager);

    data.custom_data.sky_texture = Some(texture);
    // Load the default shader
    let default_shader = Shader::default()
        .load_shader(
            vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\default.frsh.glsl"],
            data.asset_manager,
        )
        .unwrap()
        .cache(data.asset_manager);

    // Load the wireframe shader
    system.wireframe_shader = Shader::default()
        .load_shader(
            vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\others\\wireframe.frsh.glsl"],
            data.asset_manager,
        )
        .unwrap();
    // Default material
    system.default_material = Material::new("Default Material", &mut data.asset_manager).set_shader(default_shader);
}
fn system_prefire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, system.framebuffer);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    /*
    // Update the default values for each shader that exists in the shader cacher
    for shader in data.shader_cacher.1.objects.iter() {
        // Set the shader arguments
        shader.use_shader();
        shader.set_f32("delta_time", &(data.time_manager.delta_time as f32));
        shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
        //shader.set_vec2f32("resolution", &(data.custom_data.window.dimensions.into()));
    }
    */

    // Change the debug view
    if data.input_manager.map_pressed("change_debug_view") {
        system.debug_view += 1;
        system.debug_view %= 5;
    }
    // Enable / Disable wireframe
    if data.input_manager.map_pressed("toggle_wireframe") {
        system.wireframe = !system.wireframe;
    }
}
fn system_postfire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();
    let dimensions = data.custom_data.window.dimensions;
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let camera_transform = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().clone();
    let camera_position = camera_transform.position;
    let camera = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
    let vp_m = camera.projection_matrix * camera.view_matrix;
    // Draw the debug primitives
    for (i, primitive) in data.debug.renderer.primitives.iter().enumerate() {
        let (renderer, model_matrix) = data.debug.renderer.renderers.get(i).unwrap();
        let material = &renderer.material;
        let gpu_data = &renderer.gpu_data;
        let indices_count = renderer.model.triangles.len() as i32;
        system.draw_normal(
            material,
            gpu_data,
            indices_count,
            data,
            camera_position,
            &camera.projection_matrix,
            &camera.view_matrix,
            model_matrix,
        );
    }

    // Draw the volumetric stuff
    system
        .volumetric
        .calculate_volumetric(camera.projection_matrix, camera_transform.rotation, camera_transform.position, camera.clip_planes);
    // Update the frame stats texture
    system.frame_stats.update_texture(data.time_manager, &data.entity_manager.entities);
    // Draw the normal primitives
    let shader = system.quad_renderer.material.shader.as_ref().unwrap();
    shader.use_shader();
    shader.set_vec2i32("resolution", &(dimensions.into()));
    shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
    shader.set_vec2f32("nf_planes", &veclib::Vector2::new(camera.clip_planes.0, camera.clip_planes.1));
    shader.set_vec3f32("directional_light_dir", &data.custom_data.light_dir);
    // Textures
    shader.set_t2d("diffuse_texture", &system.diffuse_texture, 0);
    shader.set_t2d("normals_texture", &system.normals_texture, 1);
    shader.set_t2d("position_texture", &system.position_texture, 2);
    shader.set_t2d("depth_texture", &system.depth_texture, 3);
    shader.set_t2d("default_sky_gradient", data.custom_data.sky_texture.as_ref().unwrap(), 5);
    let vp_m = camera.projection_matrix * (veclib::Matrix4x4::from_quaternion(&camera_transform.rotation));
    shader.set_mat44("custom_vp_matrix", &vp_m);
    // Other params
    shader.set_vec3f32("camera_pos", &camera_transform.position);
    shader.set_i32("debug_view", &(system.debug_view as i32));
    shader.set_t2d("frame_stats", &system.frame_stats.texture, 6);
    // Render the screen quad
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::BindVertexArray(system.quad_renderer.gpu_data.vertex_array_object);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, system.quad_renderer.gpu_data.element_buffer_object);
        gl::DrawElements(gl::TRIANGLES, system.quad_renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
    }
}
fn entity_added(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    // Make sure we create the OpenGL data for this entity's model
    rc.refresh_model();
    let transform = entity.get_component_mut::<components::Transform>(&mut data.component_manager).unwrap();
    transform.update_matrix();
}
fn entity_removed(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    let i = Instant::now();
    // Dispose the model when the entity gets destroyed
    rc.dispose_model();
    // Dispose of a complex model if it exists
    match rc.multi_material.as_mut() {
        Some(x) => {
            // Dispose
            x.dispose_models();
        }
        None => {}
    }
}
fn entity_update(system_data: &mut SystemData, entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    let system = system_data.cast::<CustomData>().unwrap();
    // Get the camera stuff
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let camera_data = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
    let view_matrix: veclib::Matrix4x4<f32> = camera_data.view_matrix;
    let projection_matrix: veclib::Matrix4x4<f32> = camera_data.projection_matrix;
    let camera_position: veclib::Vector3<f32> = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().position;

    let model_matrix: veclib::Matrix4x4<f32> = components.get_component::<components::Transform>(data.component_manager).unwrap().matrix;
    let rc = components.get_component::<Renderer>(data.component_manager).unwrap();
    // Should we render in wireframe?
    let wireframe = system.wireframe && rc.flags.contains(RendererFlags::WIREFRAME);
    match rc.multi_material.as_ref() {
        Some(mm_renderer) => {
            // This is a Multi Material renderer
            system.draw_multi_material(mm_renderer, wireframe, data, camera_position, &projection_matrix, &view_matrix, &&model_matrix);
        }
        None => {
            if wireframe {
                system.draw_wireframe(&rc.gpu_data, rc.model.triangles.len() as i32, &projection_matrix, &view_matrix, &model_matrix);
            } else {
                system.draw_normal(
                    &rc.material,
                    &rc.gpu_data,
                    rc.model.triangles.len() as i32,
                    data,
                    camera_position,
                    &projection_matrix,
                    &view_matrix,
                    &model_matrix,
                );
            }
        }
    }
}