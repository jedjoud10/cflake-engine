use crate::engine::core::defaults::components::{components, *};

use crate::engine::rendering::renderer::{Renderer, RendererFlags};
use crate::engine::rendering::shader::Shader;


use crate::engine::core::ecs::{
    entity::Entity,
    system::System,
    system_data::{SystemData, SystemEventData},
};

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
        system_data.link_component::<components::Sky>(data.component_manager);
        system_data.link_component::<transforms::Position>(data.component_manager);
        system_data.link_component::<transforms::Scale>(data.component_manager);

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
        rc.load_textures(
            vec!["textures\\sky_gradient2.png"],
            &mut data.texture_cacher,
            &mut data.resource_manager,
        );
		rc.flags.remove(RendererFlags::Wireframe);
        // Make the skysphere inside out, so we can see the insides only
        rc.model.flip_triangles();
        sky.link_component::<Renderer>(&mut data.component_manager, rc).unwrap();
        sky.link_default_component::<transforms::Position>(&mut data.component_manager).unwrap();
        sky.link_component::<transforms::Rotation>(
            &mut data.component_manager,
            transforms::Rotation {
                rotation: glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    90.0_f32.to_radians(),
                    0.0,
                    0.0,
                ),
            },
        ).unwrap();
        sky.link_component::<transforms::Scale>(
            &mut data.component_manager,
            transforms::Scale { scale: 900.0 },
        ).unwrap();
        sky.link_default_component::<components::Sky>(&mut data.component_manager).unwrap();
        // Update the custom data
        data.custom_data.sky_component_id = sky
            .get_global_component_id::<components::Sky>(&mut data.component_manager)
            .unwrap();
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
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData) {
        // Set the position of the sky sphere to always be the camera
        let position = data
            .entity_manager
            .get_entity(data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<transforms::Position>(data.component_manager)
            .unwrap()
            .position;
        *entity
            .get_component_mut::<transforms::Position>(data.component_manager)
            .unwrap()
            .position = *position;
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
