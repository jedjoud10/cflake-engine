use main::assets::*;
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
    // Pre load the resources
    let cacher = &mut world.asset_manager.asset_cacher;
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\passthrough.vrsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\screen.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\volumetric\\volumetric.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.vrsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\others\\wireframe.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\volumetric\\volumetric_screen.cmpt.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\others\\hashes.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_elem.vrsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_panel.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.vrsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\models\\screen_quad.mdl3d", cacher);
    preload_asset!(".\\resources\\defaults\\models\\sphere.mdl3d", cacher);
    preload_asset!(".\\resources\\defaults\\models\\quad.mdl3d", cacher);
    preload_asset!(".\\resources\\defaults\\models\\cube.mdl3d", cacher);
    preload_asset!(".\\resources\\defaults\\fonts\\default_font.font", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\noise.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\erosion.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\data.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\sdf.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\color_voxel.cmpt.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain_triplanar.frsh.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel.func.glsl", cacher);
    preload_asset!(".\\resources\\defaults\\textures\\sky_gradient.png", cacher);
    preload_asset!(".\\resources\\defaults\\textures\\missing_texture.png", cacher);

    // ----Load the default systems----
    // Create the custom data
    let mut data: WorldData = WorldData {
        entity_manager: &mut world.entity_manager,
        component_manager: &mut world.component_manager,
        ui_manager: &mut world.ui_manager,
        input_manager: &mut world.input_manager,
        asset_manager: &mut world.asset_manager,
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
    let compute = Shader::new(
        vec!["defaults\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl"],
        data.asset_manager,
        Some(AdditionalShader::Compute(ComputeShader::default())),
        Some(vec!["defaults\\shaders\\voxel_terrain\\voxel.func.glsl"]),
    )
    .unwrap();

    let color_compute = Shader::new(
        vec!["defaults\\shaders\\voxel_terrain\\color_voxel.cmpt.glsl"],
        data.asset_manager,
        Some(AdditionalShader::Compute(ComputeShader::default())),
        Some(vec!["defaults\\shaders\\voxel_terrain\\voxel.func.glsl"]),
    )
    .unwrap();

    // The terrain shader
    let terrain_shader = Shader::new(
        vec![
            "defaults\\shaders\\rendering\\default.vrsh.glsl",
            "defaults\\shaders\\voxel_terrain\\terrain_triplanar.frsh.glsl",
        ],
        data.asset_manager,
        None,
        None,
    )
    .unwrap()
    .cache(&mut data.asset_manager);
    // Material
    let material = Material::new("Terrain material", &mut data.asset_manager)
        .set_shader(terrain_shader)
        .load_diffuse("defaults\\textures\\missing_texture.png", None, &mut data.asset_manager)
        .set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE * 0.7))
        .0;
    let bound_materials = vec![
        material.instantiate(data.instance_manager),
        material
            .instantiate(data.instance_manager)
            .set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE * 0.02))
            .0,
    ];
    terrain_entity
        .link_component::<components::TerrainData>(data.component_manager, components::TerrainData::new(compute, color_compute, OCTREE_DEPTH, bound_materials))
        .unwrap();
    data.entity_manager.add_entity_s(terrain_entity);

    // Template entity
    let mut cube = Entity::new("Cube");
    cube.link_default_component::<components::Transform>(data.component_manager).unwrap();
    let model = Model::asset_load_easy("defaults\\models\\cube.mdl3d", &data.asset_manager.asset_cacher).unwrap();
    let m = Material::new("Cube material", data.asset_manager);
    let renderer = Renderer::new().set_model(model).set_material(m);
    cube.link_component::<Renderer>(data.component_manager, renderer).unwrap();
    data.entity_manager.add_entity_s(cube);
}
