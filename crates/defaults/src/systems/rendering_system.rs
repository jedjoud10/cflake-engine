use super::super::components;
use ecs::{Entity, FilteredLinkedComponents};
use gl;
use math;
use rendering::{Material, MaterialFlags, Model, Renderer, RendererFlags, Shader, ShaderArg, Texture2D, TextureShaderAccessType, Window};
use resources::LoadableResource;
use std::ptr::null;
use world_data::WorldData;
use systems::{InternalSystemData, System, SystemData, SystemEventType};
use ui::Root;

#[derive(Default)]
pub struct CustomData {
    pub framebuffer: u32,
    // The frame buffer textures
    pub diffuse_texture: Texture2D,
    pub normals_texture: Texture2D,
    pub position_texture: Texture2D,
    pub emissive_texture: Texture2D,
    pub depth_stencil_texture: Texture2D,
    pub debug_view: u16,
    pub wireframe: bool,
    wireframe_shader_name: String,
    // The renderer for the screen quad
    quad_renderer: Renderer,
}
crate::impl_custom_system_data!(CustomData);

// Draw functions 
impl CustomData {
    // Read the set uniforms from a renderer's ShaderUniformSetter and update the shader
    fn set_uniforms_from_custom_setter(&self, shader: &Shader, renderer: &Renderer) {
        // Use the shader first
        shader.use_shader();
        // Loop over all the set uniforms
        for (name, data) in renderer.material.as_ref().unwrap().uniform_setter.uniforms.iter() {
            // Now it's the painful part
            match data {
                ShaderArg::F32(v) => shader.set_f32(name, v),
                ShaderArg::I32(v) => shader.set_i32(name, v),
                ShaderArg::V2F32(v) => shader.set_vec2f32(name, v),
                ShaderArg::V3F32(v) => shader.set_vec3f32(name, v),
                ShaderArg::V4F32(v) => shader.set_vec4f32(name, v),
                ShaderArg::V2I32(v) => shader.set_vec2i32(name, v),
                ShaderArg::V3I32(v) => shader.set_vec3i32(name, v),
                ShaderArg::V4I32(v) => shader.set_vec4i32(name, v),
                ShaderArg::MAT44(v) => shader.set_mat44(name, v),
            }
        }
    }
    // Create the quad that will render the render buffer
    fn create_screen_quad(&mut self, data: &mut WorldData) {
        let mut quad_renderer_component = Renderer::default();
        quad_renderer_component.model = Model::new().from_path("defaults\\models\\screen_quad.mdl3d", data.resource_manager).unwrap();
        // Create the screen quad material
        let material: Material = Material::default().set_shader(
            Shader::new(
                vec!["defaults\\shaders\\rendering\\passthrough.vrsh.glsl", "defaults\\shaders\\rendering\\screen.frsh.glsl"],
                &mut data.resource_manager,
                &mut data.shader_cacher,
                None,
            )
            .1
            .as_str(),
        );
        let mut quad_renderer_component = quad_renderer_component.set_material(material);
        quad_renderer_component.refresh_model();
        self.quad_renderer = quad_renderer_component;
    }
    // Bind a specific texture attachement to the frame buffer
    fn bind_attachement(attachement: u32, texture: &Texture2D) {
        unsafe {
            // Default target, no multisamplind
            let target: u32 = gl::TEXTURE_2D;
            gl::BindTexture(target, texture.internal_texture.id);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.internal_texture.id, 0);
        }
    }
    // Setup all the settings for opengl like culling and the clear color
    fn setup_opengl(&mut self, data: &mut WorldData) {
        let dimensions = data.custom_data.window.dimensions;
        // Initialize OpenGL
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Viewport(0, 0, dimensions.x as i32, dimensions.y as i32);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
        }

        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            // Create the diffuse render texture
            self.diffuse_texture = Texture2D::new()
                .set_dimensions_vec2(dimensions)
                .set_idf(gl::RGB, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the normals render texture
            self.normals_texture = Texture2D::new()
                .set_dimensions_vec2(dimensions)
                .set_idf(gl::RGB8_SNORM, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the position render texture
            self.position_texture = Texture2D::new()
                .set_dimensions_vec2(dimensions)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the emissive render texture
            self.emissive_texture = Texture2D::new()
                .set_dimensions_vec2(dimensions)
                .set_idf(gl::RGB16F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the depth-stencil render texture
            self.depth_stencil_texture = Texture2D::new()
                .set_dimensions_vec2(dimensions)
                .set_idf(gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8)
                .generate_texture(Vec::new());
            // Bind the color texture to the color attachement 0 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT0, &self.diffuse_texture);
            // Bind the normal texture to the color attachement 1 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT1, &self.normals_texture);
            // Bind the position texture to the color attachement 2 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT2, &self.position_texture);
            // Bind the emissive texture to the color attachement 3 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT3, &self.emissive_texture);
            // Bind the depth/stenicl texture to the color attachement depth-stencil of the frame buffer
            Self::bind_attachement(gl::DEPTH_STENCIL_ATTACHMENT, &self.depth_stencil_texture);

            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
                gl::COLOR_ATTACHMENT3,
                gl::COLOR_ATTACHMENT4,
            ];
            // Set the frame buffer attachements
            gl::DrawBuffers(attachements.len() as i32, attachements.as_ptr() as *const u32);

            // Check if the frame buffer is okay
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
            } else {
                panic!("Framebuffer has failed initialization! Error: '{}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Setup the debug renderer
        data.debug.renderer.setup_debug_renderer(data.resource_manager, data.shader_cacher);
    }
    // Draw an entity normally
    fn draw_normal(
        &self,
        renderer: &Renderer,
        data: &WorldData,
        camera_position: veclib::Vector3<f32>,
        projection_matrix: &veclib::Matrix4x4<f32>,
        view_matrix: &veclib::Matrix4x4<f32>,
        model_matrix: &veclib::Matrix4x4<f32>,
    ) {
        // Default material just in case
        let default_material = Material {
            shader_name: data.shader_cacher.1.id_get_object(0).unwrap().name.clone(),
            ..Material::default()
        };
        // Get the material for this entity
        let material = match renderer.material.as_ref() {
            Some(mat) => mat,
            None => &default_material,
        };
        // Shader name
        let shader_name = match material.shader_name.as_str() {
            "" => default_material.shader_name.clone(),
            a => a.to_string(),
        };

        // Load the shader
        let shader = data.shader_cacher.1.get_object(&shader_name).unwrap();
        // Use the shader, and update any uniforms
        shader.use_shader();
        // Calculate the mvp matrix
        let mvp_matrix: veclib::Matrix4x4<f32> = *projection_matrix * *view_matrix * *model_matrix;

        // Pass the MVP and the model matrix to the shader
        shader.set_mat44("mvp_matrix", &mvp_matrix);
        shader.set_mat44("model_matrix", model_matrix);
        shader.set_mat44("view_matrix", view_matrix);
        shader.set_vec3f32("view_pos", &camera_position);
        shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));

        // Check if we already loaded the default textures or not
        if material.diffuse_tex_id.is_none() || material.normal_tex_id.is_none() {
            // Did not load all the default textures!
            println!("Did not load all the default textures for material {}!", &material.material_name);
        }
        shader.set_t2d("diffuse_tex", data.texture_cacher.id_get_object(material.diffuse_tex_id.unwrap()).unwrap(), gl::TEXTURE0);
        shader.set_t2d("normals_tex", data.texture_cacher.id_get_object(material.normal_tex_id.unwrap()).unwrap(), gl::TEXTURE1);

        // Set the custom uniforms
        self.set_uniforms_from_custom_setter(shader, renderer);

        // Draw normally
        if renderer.gpu_data.initialized {
            // Enable / Disable vertex culling
            if material.flags.contains(MaterialFlags::DOUBLE_SIDED) {
                unsafe {
                    gl::Disable(gl::CULL_FACE);
                }
            } else {
                unsafe {
                    gl::Enable(gl::CULL_FACE);
                }
            }
            unsafe {
                // Actually draw the array
                gl::BindVertexArray(renderer.gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, renderer.gpu_data.element_buffer_object);
                gl::DrawElements(gl::TRIANGLES, renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
            }
        }
    }
    // Draw a wireframe entity
    fn draw_wireframe(
        &self,
        renderer: &Renderer,
        data: &WorldData,
        projection_matrix: &veclib::Matrix4x4<f32>,
        view_matrix: &veclib::Matrix4x4<f32>,
        model_matrix: &veclib::Matrix4x4<f32>,
    ) {
        if renderer.gpu_data.initialized && renderer.flags.contains(RendererFlags::WIREFRAME) {
            let wireframe_shader = data.shader_cacher.1.get_object(&self.wireframe_shader_name).unwrap();
            wireframe_shader.use_shader();
            // Calculate the mvp matrix
            let mvp_matrix: veclib::Matrix4x4<f32> = *projection_matrix * *view_matrix * *model_matrix;
            wireframe_shader.set_mat44("mvp_matrix", &mvp_matrix);
            wireframe_shader.set_mat44("model_matrix", model_matrix);
            wireframe_shader.set_mat44("view_matrix", view_matrix);
            unsafe {
                // Set the wireframe rendering
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                gl::Disable(gl::CULL_FACE);

                gl::BindVertexArray(renderer.gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, renderer.gpu_data.element_buffer_object);
                gl::DrawElements(gl::TRIANGLES, renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());

                // Reset the wireframe settings
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::Enable(gl::CULL_FACE);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }
        }
    }
}

// Events 
fn system_enabled(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();

    // Create the screen quad
    system.create_screen_quad(data);

    // Then setup opengl and the render buffer
    system.setup_opengl(data);

    // Load the wireframe shad
    let wireframe_shader_name = Shader::new(
        vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\others\\wireframe.frsh.glsl"],
        &mut data.resource_manager,
        &mut data.shader_cacher,
        None,
    )
    .1;
    system.wireframe_shader_name = wireframe_shader_name;
}
fn system_prefire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, system.framebuffer);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    // Update the default values for each shader that exists in the shader cacher
    for shader in data.shader_cacher.1.objects.iter() {
        // Set the shader arguments
        shader.set_f32("delta_time", &(data.time_manager.delta_time as f32));
        shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
        shader.set_vec2f32("resolution", &(data.custom_data.window.dimensions.into()));
    }
    // Change the debug view
    if data.input_manager.map_pressed("change_debug_view") {
        system.debug_view += 1;
        system.debug_view %= 4;
    }
    // Enable / Disable wireframe
    if data.input_manager.map_pressed("toggle_wireframe") {
        system.wireframe = !system.wireframe;
    }
}
fn system_postfire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast::<CustomData>().unwrap();
    let dimensions = data.custom_data.window.dimensions;
    // At the end of each frame, disable the depth test and render the debug objects
    let vp_matrix: veclib::Matrix4x4<f32>;
    let camera_position: veclib::Vector3<f32>;
    // Get the (projection * view) matrix
    {
        let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
        let camera_data = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
        let projection_matrix = camera_data.projection_matrix;
        let view_matrix = camera_data.view_matrix;
        vp_matrix = projection_matrix * view_matrix;
        camera_position = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().position;
    }
    // Draw the debug primitives
    data.debug.renderer.draw_debug(&vp_matrix, &data.shader_cacher.1);

    // Draw the normal primitives
    let shader = data.shader_cacher.1.get_object(&system.quad_renderer.material.as_ref().unwrap().shader_name).unwrap();
    shader.use_shader();
    shader.set_t2d("diffuse_texture", &system.diffuse_texture, gl::TEXTURE0);
    shader.set_t2d("normals_texture", &system.normals_texture, gl::TEXTURE1);
    shader.set_t2d("position_texture", &system.position_texture, gl::TEXTURE2);
    shader.set_t2d("emissive_texture", &system.emissive_texture, gl::TEXTURE3);
    shader.set_vec2i32("resolution", &(dimensions.into()));
    shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
    // Sky params
    shader.set_vec3f32("directional_light_dir", &veclib::Vector3::new(0.0, 1.0, 0.0));    
    let sky_component = data
        .entity_manager
        .get_entity(data.custom_data.sky_entity_id)
        .unwrap()
        .get_component::<components::Sky>(data.component_manager)
        .unwrap();

    // Set the sky gradient
    shader.set_t2d(
        "default_sky_gradient",
        data.texture_cacher.id_get_object(sky_component.sky_gradient_texture_id).unwrap(),
        gl::TEXTURE4,
    );
    

    // Other params
    shader.set_vec3f32("view_pos", &camera_position);
    shader.set_i32("debug_view", &(system.debug_view as i32));
    // Render the screen quad
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::BindVertexArray(system.quad_renderer.gpu_data.vertex_array_object);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, system.quad_renderer.gpu_data.element_buffer_object);
        gl::DrawElements(gl::TRIANGLES, system.quad_renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
    }
}
fn entity_added(system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    // Make sure we create the OpenGL data for this entity's model
    rc.refresh_model();
    let transform = entity.get_component_mut::<components::Transform>(&mut data.component_manager).unwrap();
    transform.update_matrix();
}
fn entity_removed(system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    // Dispose the model when the entity gets destroyed
    rc.dispose_model();
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

    // Draw the entity normally
    if system.wireframe {
        system.draw_wireframe(rc, data, &projection_matrix, &view_matrix, &model_matrix);
    } else {
        system.draw_normal(rc, data, camera_position, &projection_matrix, &view_matrix, &model_matrix);
    }
}
// Create the rendering system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Transform>(data.component_manager).unwrap();
    system.link_component::<rendering::Renderer>(data.component_manager).unwrap();
    // Some input events
    data.input_manager.bind_key(input::Keys::F, "toggle_wireframe", input::MapType::Button);    
    data.input_manager.bind_key(input::Keys::F3, "change_debug_view", input::MapType::Button);
    // Attach the events
    system.event(SystemEventType::SystemEnabled(system_enabled));
    system.event(SystemEventType::SystemPrefire(system_prefire));
    system.event(SystemEventType::SystemPostfire(system_postfire));
    system.event(SystemEventType::EntityAdded(entity_added));
    system.event(SystemEventType::EntityRemoved(entity_removed));
    system.event(SystemEventType::EntityUpdate(entity_update));
    // Attach the custom system data
    system.custom_data(CustomData::default());
    system
}