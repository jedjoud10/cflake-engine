use main::defaults::components;
use main::defaults::systems;
use main::ecs::*;
use main::others::Instance;
use main::rendering::*;
use main::world_data::*;
use main::*;
fn main() {
    // Load up the engine
    main::start("DevJed", "DevGame", world_initialized);
}
pub fn world_initialized(world: &mut World) {
    // ----Load the default systems----
    // Create the custom data
    let mut data: WorldData = WorldData {
        entity_manager: &mut world.entity_manager,
        component_manager: &mut world.component_manager,
        ui_manager: &mut world.ui_manager,
        input_manager: &mut world.input_manager,
        shader_cacher: &mut world.shader_cacher,
        texture_cacher: &mut world.texture_cacher,
        resource_manager: &mut world.resource_manager,
        time_manager: &mut world.time_manager,
        debug: &mut world.debug,
        custom_data: &mut world.custom_data,
        instance_manager: &mut world.instance_manager,
    };

    // Load the rendering system
    let mut rendering_system = systems::rendering_system::system(&mut data);
    rendering_system.enable(&mut data);
    world.system_manager.add_system(rendering_system);
    // Load the camera system
    let mut camera_system = systems::camera_system::system(&mut data);
    camera_system.enable(&mut data);
    world.system_manager.add_system(camera_system);
    // Load the sky system
    let mut sky_system = systems::sky_system::system(&mut data);
    sky_system.enable(&mut data);
    world.system_manager.add_system(sky_system);
    // Load the default UI system
    let mut ui_system = systems::ui_system::system(&mut data);
    ui_system.enable(&mut data);
    world.system_manager.add_system(ui_system);
    // Load the default command system
    let mut command_system = systems::command_system::system(&mut data);
    command_system.enable(&mut data);
    world.system_manager.add_system(command_system);
    // Load the terrain system
    let mut terrain_system = systems::terrain_system::system(&mut data);
    terrain_system.enable(&mut data);
    world.system_manager.add_system(terrain_system);
    // ----Load the entities----
    // Create a camera entity

    let mut camera = Entity::new("Default Camera");
    camera.link_default_component::<components::Transform>(data.component_manager).unwrap();
    camera.link_default_component::<components::Physics>(data.component_manager).unwrap();
    camera.link_default_component::<components::Camera>(data.component_manager).unwrap();

    // Make it the default camera
    data.custom_data.main_camera_entity_id = data.entity_manager.add_entity_s(camera);

    // Create the terrain entity
    let mut terrain_entity = Entity::new("Default Terrain");
    const OCTREE_DEPTH: u8 = 7;

    // Load the compute shaders
    let compute_name = Shader::new(
        vec!["user\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl"],
        data.resource_manager,
        data.shader_cacher,
        Some(AdditionalShader::Compute(ComputeShader::default())),
    )
    .1;

    let color_compute_name = Shader::new(
        vec!["user\\shaders\\voxel_terrain\\color_voxel.cmpt.glsl"],
        data.resource_manager,
        data.shader_cacher,
        Some(AdditionalShader::Compute(ComputeShader::default())),
    )
    .1;

    // The terrain shader
    let terrain_shader = Shader::new(
        vec![
            "defaults\\shaders\\rendering\\default.vrsh.glsl",
            "defaults\\shaders\\voxel_terrain\\terrain_triplanar.frsh.glsl",
        ],
        data.resource_manager,
        data.shader_cacher,
        None,
    )
    .1;
    // Material
    let material = Material::new("Terrain material")
        .set_shader(&terrain_shader)
        .resource_load_textures(vec![Some("defaults\\textures\\rock_diffuse.png"), Some("defaults\\textures\\rock_normal.png")], data.texture_cacher, data.resource_manager)
        .unwrap()
        .load_default_textures(data.texture_cacher);
    let material_inst = material
        .instantiate(data.instance_manager)
        .set_uniform("uv_scale", ShaderArg::V2F32(veclib::Vector2::ONE * 0.2));
    terrain_entity
        .link_component::<components::TerrainData>(
            data.component_manager,
            components::TerrainData::new(material_inst, compute_name, color_compute_name, OCTREE_DEPTH),
        )
        .unwrap();
    data.entity_manager.add_entity_s(terrain_entity);
}
