use crate::engine::core::defaults::components;

use crate::engine::core::ecs::component::FilteredLinkedComponents;
use crate::engine::rendering::renderer::{Renderer, RendererFlags};
use crate::engine::rendering::shader::Shader;

use crate::engine::core::ecs::{
    entity::Entity,
    system::System,
    system_data::{SystemData, SystemEventData},
};
use crate::engine::rendering::texture::{Texture, TextureWrapping};

#[derive(Default)]
pub struct SkySystem {
    pub system_data: SystemData,
}

impl System for SkySystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        let system_data = self.get_system_data_mut();
        system_data.link_component::<components::Sky>(data.component_manager).unwrap();
        system_data.link_component::<components::Transform>(data.component_manager).unwrap();

        // Create the sky entity
        let mut sky = Entity::new("Sky");
        // Use a custom shader
        let sky_shader_name = Shader::new(
            vec!["shaders\\default.vrsh.glsl", "shaders\\sky.frsh.glsl"],
            &mut data.resource_manager,
            &mut data.shader_cacher,
        )
        .1;
        let mut rc = Renderer::default();
        rc.load_model("models\\sphere.mdl3d", &mut data.resource_manager);
        rc.shader_name = sky_shader_name;

        // The texture that will be used as gradient
        let cached_texture_id = Texture::new()
            .enable_mipmaps()
            .set_wrapping_mode(TextureWrapping::ClampToEdge)
            .load_texture("textures\\sky_gradient.png", data.resource_manager, data.texture_cacher)
            .unwrap()
            .1;
        rc.load_textures(vec![cached_texture_id], &mut data.texture_cacher);
        rc.flags.remove(RendererFlags::WIREFRAME);
        // Make the skysphere inside out, so we can see the insides only
        rc.model.flip_triangles();
        sky.link_component::<Renderer>(data.component_manager, rc).unwrap();
        sky.link_default_component::<components::AABB>(data.component_manager).unwrap();
        sky.link_component::<components::Transform>(
            data.component_manager,
            components::Transform {
                position: glam::Vec3::ZERO,
                scale: glam::Vec3::ONE * 9000.0,
                ..components::Transform::default()
            },
        ).unwrap();
        sky.link_component::<components::Sky>(
            &mut data.component_manager,
            components::Sky {
                sky_gradient_texture_id: cached_texture_id,
            },
        )
        .unwrap();
        // Update the custom data
        data.custom_data.sky_entity_id = sky.entity_id;
        data.entity_manager.add_entity_s(sky);
    }

    // Update the sun rotation
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        data.custom_data.sun_rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            data.time_manager.seconds_since_game_start as f32 / 4.0,
            data.time_manager.seconds_since_game_start as f32 / 4.0,
            data.time_manager.seconds_since_game_start as f32 / 4.0,
        );
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {
        // Set the position of the sky sphere to always be the camera's position
        let position = data
            .entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<components::Transform>(data.component_manager)
            .unwrap()
            .position;
        components.get_component_mut::<components::Transform>(data.component_manager).unwrap().position = position;
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
