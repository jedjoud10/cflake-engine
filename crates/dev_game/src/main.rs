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
    main::start("DevJed", "DevGame", assets_preload, world_initialized);
}
pub fn assets_preload() {
    // -----Pre-load the game assets here-----
}
pub fn world_initialized(world: &mut World) {
    // ----Load the default systems----
    // Create the custom data
    let mut data: WorldData = WorldData {
        entity_manager: &mut world.entity_manager,
        component_manager: &mut world.component_manager,
        ui_manager: &mut world.ui_manager,
        input_manager: &mut world.input_manager,
        time_manager: &mut world.time_manager,
        debug: &mut world.debug,
        custom_data: &mut world.custom_data,
        instance_manager: &mut world.instance_manager,
    };
    // ----Load the entities----
    // Create a camera entity

    let mut camera = Entity::new("Default Camera");
    camera.link_default_component::<components::Transform>(data.component_manager).unwrap();
    camera.link_default_component::<components::Physics>(data.component_manager).unwrap();
    camera
        .link_component::<components::Camera>(data.component_manager, components::Camera::new(90.0, 3.0, 200000.0))
        .unwrap();

    // Make it the default camera
    data.custom_data.main_camera_entity_id = data.entity_manager.add_entity_s(camera);
    let model = pipec::model(assets::assetc::dload("defaults\\models\\cube.mdl3d").unwrap());
    let default_material = Material::new("Default material");
    for x in 0..5 {        
        let mut entity = Entity::new("Test");
        entity.link_component::<components::Transform>(data.component_manager, components::Transform::default().with_position(veclib::Vector3::<f32>::new(x as f32, 0.0, 0.0))).unwrap();
        let renderer = components::Renderer::default().set_model(model.clone()).set_material(default_material.clone());
        entity.link_component::<components::Renderer>(data.component_manager, renderer).unwrap();
        data.entity_manager.add_entity_s(entity);    
    }
    
    // Create the terrain entity
    let mut terrain_entity = Entity::new("Default Terrain");
    // The terrain shader
    let terrain_shader = pipec::shader(Shader::default()
        .load_shader(
            vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl"],
        ).unwrap());
    // Material
    let mut material = Material::new("Terrain material").set_shader(terrain_shader);
    material.uniforms.set_f32("normals_strength", 2.0);
    material.uniforms.set_vec2f32("uv_scale", veclib::Vector2::ONE * 0.7);
    /*
    let texture = Texture::create_texturearray(
        Some(TextureLoadOptions {
            filter: TextureFilter::Nearest,
            ..TextureLoadOptions::default()
        }),
        vec!["defaults\\textures\\rock_diffuse.png", "defaults\\textures\\missing_texture.png"],
        data.asset_manager,
        256,
        256,
    )
    .object_cache_load("fart", &mut data.asset_manager.object_cacher);
    let texture2 = Texture::create_texturearray(
        Some(TextureLoadOptions {
            filter: TextureFilter::Nearest,
            ..TextureLoadOptions::default()
        }),
        vec!["defaults\\textures\\rock_normal.png", "defaults\\textures\\missing_texture.png"],
        data.asset_manager,
        256,
        256,
    )
    .object_cache_load("fasdfart", &mut data.asset_manager.object_cacher);
    let bound_materials = vec![material
    .instantiate(data.instance_manager)
    .set_uniform("diffuse_textures", Uniform::Texture2DArray(texture, 0))
    .set_uniform("normals_textures", Uniform::Texture2DArray(texture2, 1))
    .set_uniform("material_id", Uniform::I32(0))];
    */
    let settings = terrain::TerrainSettings {
        octree_depth: 10,
        material,
        voxel_generator_interpreter: terrain::interpreter::Interpreter::new_pregenerated(),
    };
    terrain_entity
        .link_component::<components::TerrainData>(data.component_manager, components::TerrainData::new(settings))
        .unwrap();
    data.entity_manager.add_entity_s(terrain_entity);
}
