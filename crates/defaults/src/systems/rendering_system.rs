use super::super::components;
use assets::{Asset, Object};
use ecs::{Entity, FilteredLinkedComponents};
use gl;
use rendering::{Material, MaterialFlags, Model, ModelDataGPU, MultiMaterialRenderer, Renderer, RendererFlags, Shader, Texture, TextureType, TextureWrapping, Volumetric};
use std::{
    ffi::{c_void, CString},
    ptr::null,
    time::Instant,
};
use systems::{InternalSystemData, System, SystemData, SystemEventType};
use veclib::Swizzable;
use world_data::WorldData;

#[derive(Default)]
pub struct CustomData {
    pub framebuffer: u32,
    // The frame buffer textures
    pub diffuse_texture: Texture,
    pub normals_texture: Texture,
    pub position_texture: Texture,
    pub depth_texture: Texture,
    pub debug_view: u16,
    pub wireframe: bool,
    wireframe_shader: Shader,
    default_material: Material,
    // Volumetric renderer stuff
    pub volumetric: Volumetric,
    // The renderer for the screen quad
    quad_renderer: Renderer,
}
crate::impl_custom_system_data!(CustomData);

// Draw functions
impl CustomData {
    // Create the quad that will render the render buffer
    fn create_screen_quad(&mut self, data: &mut WorldData) {
        let mut quad_renderer_component = Renderer::default();
        quad_renderer_component.model = Model::asset_load_easy("defaults\\models\\screen_quad.mdl3d", &mut data.asset_manager.asset_cacher).unwrap();
        // Create the screen quad material
        let material: Material = Material::default().set_shader(
            Shader::new(
                vec!["defaults\\shaders\\rendering\\passthrough.vrsh.glsl", "defaults\\shaders\\rendering\\screen.frsh.glsl"],
                &mut data.asset_manager,
                None,
                None,
            )
            .unwrap()
            .cache(data.asset_manager),
        );
        let mut quad_renderer_component = quad_renderer_component.set_material(material);
        quad_renderer_component.refresh_model();
        self.quad_renderer = quad_renderer_component;
    }
    // Bind a specific texture attachement to the frame buffer
    fn bind_attachement(attachement: u32, texture: &Texture) {
        unsafe {
            // Default target, no multisamplind
            let target: u32 = gl::TEXTURE_2D;
            gl::BindTexture(target, texture.id);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.id, 0);
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
            let dims = TextureType::Texture2D(dimensions.x, dimensions.y);
            // Create the diffuse render texture
            self.diffuse_texture = Texture::new()
                .set_dimensions(dims)
                .set_idf(gl::RGB, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new())
                .unwrap();
            // Create the normals render texture
            self.normals_texture = Texture::new()
                .set_dimensions(dims)
                .set_idf(gl::RGB8_SNORM, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new())
                .unwrap();
            // Create the position render texture
            self.position_texture = Texture::new()
                .set_dimensions(dims)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new())
                .unwrap();
            // Create the depth render texture
            self.depth_texture = Texture::new()
                .set_dimensions(dims)
                .set_idf(gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::FLOAT)
                .generate_texture(Vec::new())
                .unwrap();
            // Bind the color texture to the color attachement 0 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT0, &self.diffuse_texture);
            // Bind the normal texture to the color attachement 1 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT1, &self.normals_texture);
            // Bind the position texture to the color attachement 2 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT2, &self.position_texture);
            // Bind the depth/stenicl texture to the color attachement depth-stencil of the frame buffer
            Self::bind_attachement(gl::DEPTH_ATTACHMENT, &self.depth_texture);

            let attachements = vec![gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
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
        data.debug.renderer.setup_debug_renderer(data.asset_manager);
    }
    // Draw an entity normally
    fn draw_normal(
        &self,
        material: &Material,
        gpu_data: &ModelDataGPU,
        indices_count: i32,
        data: &WorldData,
        camera_position: veclib::Vector3<f32>,
        projection_matrix: &veclib::Matrix4x4<f32>,
        view_matrix: &veclib::Matrix4x4<f32>,
        model_matrix: &veclib::Matrix4x4<f32>,
    ) {
        // Exit early
        if !material.visible {
            return;
        }
        // Shader name
        let shader = match &material.shader {
            None => self.default_material.shader.as_ref().unwrap(),
            Some(x) => x,
        };

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

        // Set the default/custom uniforms
        for uniform in material.default_uniforms.iter() {
            let name = uniform.0.as_str();
            match &uniform.1 {
                rendering::DefaultUniform::F32(x) => shader.set_f32(name, x),
                rendering::DefaultUniform::I32(x) => shader.set_i32(name, x),
                rendering::DefaultUniform::Vec2F32(x) => shader.set_vec2f32(name, x),
                rendering::DefaultUniform::Vec3F32(x) => shader.set_vec3f32(name, x),
                rendering::DefaultUniform::Vec4F32(x) => shader.set_vec4f32(name, x),
                rendering::DefaultUniform::Vec2I32(x) => shader.set_vec2i32(name, x),
                rendering::DefaultUniform::Vec3I32(x) => shader.set_vec3i32(name, x),
                rendering::DefaultUniform::Vec4I32(x) => shader.set_vec4i32(name, x),
                rendering::DefaultUniform::Mat44F32(x) => shader.set_mat44(name, x),
                rendering::DefaultUniform::Texture2D(x, y) => panic!(),
                rendering::DefaultUniform::Texture3D(x, y) => panic!(),
            }
        }

        // Set the textures
        shader.set_t2d("diffuse_tex", material.diffuse_tex.as_ref().unwrap(), gl::TEXTURE0);
        shader.set_t2d("normals_tex", material.normal_tex.as_ref().unwrap(), gl::TEXTURE1);

        // Draw normally
        if gpu_data.initialized {
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
                gl::BindVertexArray(gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gpu_data.element_buffer_object);
                gl::DrawElements(gl::TRIANGLES, indices_count, gl::UNSIGNED_INT, null());
            }
        }
    }
    // Draw a multi material renderer
    fn draw_multi_material(
        &self,
        mm_renderer: &MultiMaterialRenderer,
        wireframe: bool,
        data: &WorldData,
        camera_position: veclib::Vector3<f32>,
        projection_matrix: &veclib::Matrix4x4<f32>,
        view_matrix: &veclib::Matrix4x4<f32>,
        model_matrix: &veclib::Matrix4x4<f32>,
    ) {
        // Loop the sub models and use them to make a sub renderer and render that separately
        for (i, (sub_model, material_id)) in mm_renderer.sub_models.iter().enumerate() {
            let material = mm_renderer.materials.get(*material_id).unwrap_or(mm_renderer.materials.get(0).unwrap());
            match mm_renderer.sub_models_gpu_data.get(i) {
                Some(gpu_data) => {
                    if wireframe {
                        self.draw_wireframe(&gpu_data, sub_model.triangles.len() as i32, projection_matrix, view_matrix, model_matrix);
                    } else {
                        self.draw_normal(
                            material,
                            &gpu_data,
                            sub_model.triangles.len() as i32,
                            data,
                            camera_position,
                            projection_matrix,
                            view_matrix,
                            model_matrix,
                        );
                    }
                }
                None => {}
            }
        }
    }
    // Draw a wireframe entity
    fn draw_wireframe(
        &self,
        gpu_data: &ModelDataGPU,
        indices_count: i32,
        projection_matrix: &veclib::Matrix4x4<f32>,
        view_matrix: &veclib::Matrix4x4<f32>,
        model_matrix: &veclib::Matrix4x4<f32>,
    ) {
        self.wireframe_shader.use_shader();
        // Calculate the mvp matrix
        let mvp_matrix: veclib::Matrix4x4<f32> = *projection_matrix * *view_matrix * *model_matrix;
        self.wireframe_shader.set_mat44("mvp_matrix", &mvp_matrix);
        self.wireframe_shader.set_mat44("model_matrix", model_matrix);
        self.wireframe_shader.set_mat44("view_matrix", view_matrix);
        unsafe {
            // Set the wireframe rendering
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::Enable(gl::LINE_SMOOTH);

            gl::BindVertexArray(gpu_data.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gpu_data.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, indices_count, gl::UNSIGNED_INT, null());

            // Reset the wireframe settings
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::Disable(gl::LINE_SMOOTH);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
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
        println!("OpenGL version; major: '{}', minor: '{}'", major, minor);
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

    // Load sky gradient texture
    let texture = Texture::new()
        .set_wrapping_mode(TextureWrapping::ClampToEdge)
        .cache_load("defaults\\textures\\sky_gradient.png", data.asset_manager);

    data.custom_data.sky_texture = Some(texture);
    // Load the default shader
    let default_shader = Shader::new(
        vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\default.frsh.glsl"],
        data.asset_manager,
        None,
        None,
    )
    .unwrap()
    .cache(data.asset_manager);

    // Load the wireframe shader
    system.wireframe_shader = Shader::new(
        vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\others\\wireframe.frsh.glsl"],
        data.asset_manager,
        None,
        None,
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
    let camera = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
    let vp_m = camera.projection_matrix * camera.view_matrix;
    // Draw the debug primitives
    data.debug.renderer.draw_debug(&vp_m);

    // Draw the volumetric stuff
    system
        .volumetric
        .calculate_volumetric(camera.projection_matrix, camera_transform.rotation, camera_transform.position, camera.clip_planes);
    // Draw the normal primitives
    let shader = system.quad_renderer.material.shader.as_ref().unwrap();
    shader.use_shader();
    shader.set_vec2i32("resolution", &(dimensions.into()));
    shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
    shader.set_vec2f32("nf_planes", &veclib::Vector2::new(camera.clip_planes.0, camera.clip_planes.1));
    shader.set_vec3f32("directional_light_dir", &data.custom_data.light_dir);
    // Textures
    shader.set_t2d("diffuse_texture", &system.diffuse_texture, gl::TEXTURE0);
    shader.set_t2d("normals_texture", &system.normals_texture, gl::TEXTURE1);
    shader.set_t2d("position_texture", &system.position_texture, gl::TEXTURE2);
    shader.set_t2d("depth_texture", &system.depth_texture, gl::TEXTURE3);
    shader.set_t2d("default_sky_gradient", data.custom_data.sky_texture.as_ref().unwrap(), gl::TEXTURE5);
    let vp_m = camera.projection_matrix * (veclib::Matrix4x4::from_quaternion(&camera_transform.rotation));
    shader.set_mat44("custom_vp_matrix", &vp_m);
    // Other params
    shader.set_vec3f32("camera_pos", &camera_transform.position);
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
// Aa frustum culling
fn entity_filter(components: &FilteredLinkedComponents, data: &WorldData) -> bool {
    let renderer = components.get_component::<Renderer>(data.component_manager).unwrap();
    let aabb = components.get_component::<components::AABB>(data.component_manager).unwrap().aabb;
    // We have an AABB, we can do the frustum culling
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let frustum = &camera_entity.get_component::<components::Camera>(data.component_manager).unwrap().frustum;
    let visible_frustum_culling = math::Intersection::frustum_aabb(&frustum, &aabb);
    return renderer.visible && visible_frustum_culling;
}

// Create the rendering system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Transform>(data.component_manager).unwrap();
    system.link_component::<rendering::Renderer>(data.component_manager).unwrap();
    system.link_component::<components::AABB>(data.component_manager).unwrap();
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
    system.event(SystemEventType::EntityFilter(entity_filter));
    // Attach the custom system data
    system.custom_data(CustomData::default());
    system
}
