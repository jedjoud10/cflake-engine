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
        asset_manager: &mut world.asset_manager,
        asset_manager: &mut world.resource_manager,
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
    let compute_id = Shader::new(
        vec!["defaults\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl"],
        data.asset_manager,
        data.shader_cacher,
        Some(AdditionalShader::Compute(ComputeShader::default())),
        Some(vec!["user\\shaders\\voxel_terrain\\voxel.func.glsl"]),
    )
    .2;

    let color_compute_id = Shader::new(
        vec!["defaults\\shaders\\voxel_terrain\\color_voxel.cmpt.glsl"],
        data.asset_manager,
        data.shader_cacher,
        Some(AdditionalShader::Compute(ComputeShader::default())),
        Some(vec!["user\\shaders\\voxel_terrain\\voxel.func.glsl"]),
    )
    .2;

    // The terrain shader
    let terrain_shader = Shader::new(
        vec![
            "defaults\\shaders\\rendering\\default.vrsh.glsl",
            "defaults\\shaders\\voxel_terrain\\terrain_triplanar.frsh.glsl",
        ],
        data.asset_manager,
        data.shader_cacher,
        None,
        None,
    )
    .1;
    // Material
    let material = Material::new("Terrain material")
        .set_shader(&terrain_shader)
        .resource_load_textures(
            vec![Some("defaults\\textures\\rock_diffuse.png"), Some("defaults\\textures\\rock_normal.png")],
            None,
            data.texture_cacher,
            data.asset_manager,
        )
        .unwrap()
        .set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE * 0.7))
        .0
        .load_default_textures(data.texture_cacher);
    let a = TextureLoadOptions {
        filter: TextureFilter::Nearest,
        ..TextureLoadOptions::default()
    };
    let bound_materials = vec![
        Some(material.instantiate(data.instance_manager)),
        Some(
            material
                .instantiate(data.instance_manager)
                .set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE * 0.02))
                .0
                .load_diffuse("user\\textures\\sandstone_cracks_diff_4k.png", Some(a), data.texture_cacher, data.asset_manager)
                .load_normal("user\\textures\\sandstone_cracks_nor_gl_4k.png", Some(a), data.texture_cacher, data.asset_manager),
        ),
    ];
    terrain_entity
        .link_component::<components::TerrainData>(
            data.component_manager,
            components::TerrainData::new(compute_id, color_compute_id, OCTREE_DEPTH, bound_materials),
        )
        .unwrap();

    // Template entity

    let mut cube = Entity::new("Cube");
    cube.link_component::<components::Transform>(data.component_manager, components::Transform::default().with_position(veclib::Vector3::new(0.0, 0.0, 0.0)))
        .unwrap();
    let model = Model::new().from_path("user\\models\\tools2.mdl3d", data.asset_manager).unwrap();
    let m = Material::new("M")
        .load_diffuse(
            "user\\textures\\palette.png",
            Some(TextureLoadOptions {
                filter: TextureFilter::Nearest,
                ..TextureLoadOptions::default()
            }),
            data.texture_cacher,
            data.asset_manager,
        )
        .load_default_textures(data.texture_cacher);
    let renderer = Renderer::new().set_model(model).set_material(m);
    cube.link_component::<Renderer>(data.component_manager, renderer).unwrap();
    data.entity_manager.add_entity_s(cube);

    data.entity_manager.add_entity_s(terrain_entity);
}
