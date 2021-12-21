use main::defaults::components;
use main::*;
fn main() {
    // Load up the engine
    main::start("DevJed", "DevGame", assets_preload, load_entities, load_systems);
}
pub fn assets_preload() {
    // -----Pre-load the game assets here-----
}
pub fn load_systems() {}
pub fn load_entities() {
    // ----Load the entities----

    // Create the default camera
    let mut linkings = ecs::ComponentLinkingGroup::new();
    linkings.link_default::<crate::components::Transform>().unwrap();
    linkings.link_default::<crate::components::Camera>().unwrap();
    // Add the camera
    let main_camera_entity_id = core::global::ecs::entity_add(ecs::Entity::new("Default Camera"), linkings)
        .immediate_result()
        .entity_id()
        .unwrap();
    core::global::main::world_data_mut(|data| data.main_camera_entity_id = main_camera_entity_id);
    let model = rendering::pipec::model(assets::assetc::dload("defaults\\models\\sphere.mdl3d").unwrap());
    for x in 0..30 {
        let mut linkings = ecs::ComponentLinkingGroup::new();
        linkings
            .link(crate::components::Transform::default().with_position(veclib::Vector3::new(0.0, 0.0, x as f32)))
            .unwrap();
        //linkings.link(crate::components::Renderer::default().set_model(model.clone()).set_material(rendering::Material::new("Test"))).unwrap();
        core::global::ecs::entity_add(ecs::Entity::new("Sphere"), linkings);
    }

    /*
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
    let default_material = Material::new("Default material").load_diffuse(
        "defaults\\textures\\rock_normal.png",
        Some(TextureLoadOptions {
            filter: TextureFilter::Nearest,
            wrapping: TextureWrapping::Repeat,
        }),
    );
    for x in 0..5 {
        let mut entity = Entity::new("Test");
        entity
            .link_component::<components::Transform>(
                data.component_manager,
                components::Transform::default().with_position(veclib::Vector3::<f32>::new(x as f32, 0.0, 0.0)),
            )
            .unwrap();
        let renderer = components::Renderer::default().set_model(model.clone()).set_material(default_material.clone());
        entity.link_component::<components::Renderer>(data.component_manager, renderer).unwrap();
        data.entity_manager.add_entity_s(entity);
    }

    // Create the terrain entity
    let mut terrain_entity = Entity::new("Default Terrain");
    // The terrain shader
    let terrain_shader = pipec::shader(
        Shader::default()
            .load_shader(vec![
                "defaults\\shaders\\rendering\\default.vrsh.glsl",
                "defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl",
            ])
            .unwrap(),
    );
    // Material
    let mut material = Material::new("Terrain material").set_shader(terrain_shader);
    material.uniforms.set_f32("normals_strength", 2.0);
    material.uniforms.set_vec2f32("uv_scale", veclib::Vector2::ONE * 0.7);
    // Create the diffuse texture array
    let texture = pipec::texturec(
        assets::cachec::cache_l(
            "terrain_diffuse_texture",
            Texture::create_texturearray(vec!["defaults\\textures\\rock_diffuse.png", "defaults\\textures\\missing_texture.png"], 256, 256)
                .apply_texture_load_options(None)
                .enable_mipmaps(),
        )
        .unwrap(),
    );
    // Create the normalmap texture array
    let texture2 = pipec::texturec(
        assets::cachec::cache_l(
            "terrain_normal_map_texture",
            Texture::create_texturearray(vec!["defaults\\textures\\rock_normal.png", "defaults\\textures\\missing_texture.png"], 256, 256)
                .apply_texture_load_options(None)
                .enable_mipmaps(),
        )
        .unwrap(),
    );
    // Assign
    let group = &mut material.uniforms;
    group.set_t2da("diffuse_textures", texture, 0);
    group.set_t2da("normals_textures", texture2, 1);
    group.set_i32("material_id", 0);
    let settings = terrain::TerrainSettings {
        octree_depth: 8,
        material,
        voxel_generator_interpreter: terrain::interpreter::Interpreter::new_pregenerated(),
    };
    terrain_entity
        .link_component::<components::TerrainData>(data.component_manager, components::TerrainData::new(settings))
        .unwrap();
    data.entity_manager.add_entity_s(terrain_entity);
    */
}
