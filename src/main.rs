use cflake_engine::*;
fn main() {
    // Load up the engine
    start("DevJed", "cflake-engine", preload_assets, init);
}
fn preload_assets() {
    // -----Pre-load the game assets here-----
    assets::preload_asset!(".\\resources\\user\\textures\\rock_diffuse.png");
    assets::preload_asset!(".\\resources\\user\\textures\\rock_normal.png");
    assets::preload_asset!(".\\resources\\user\\textures\\forrest_ground_01_diff_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\forrest_ground_01_nor_gl_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\rocks_ground_06_diff_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\rocks_ground_06_nor_gl_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\rocks_ground_08_diff_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\rocks_ground_08_nor_gl_2k.jpg");
    assets::preload_asset!(".\\resources\\user\\textures\\saber.png");
    assets::preload_asset!(".\\resources\\user\\shaders\\voxel_terrain\\voxel.func.glsl");
}
fn init(world: &mut core::World) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ecs::entity::ComponentLinkingGroup::default();
    group.link(defaults::components::Camera::new(90.0, 0.6, 1000.0)).unwrap();
    group.link_default::<defaults::components::Transform>().unwrap();
    let entity = ecs::entity::Entity::default();
    let id = ecs::entity::EntityID::new(&mut world.ecs);
    world.ecs.add_entity(entity, id, group).unwrap();
    let pipeline_ = world.pipeline.clone();
    let pipeline = pipeline_.read();
    // Create it's model
    let model = assets::assetc::dload::<rendering::basics::model::Model>("defaults\\models\\cube.mdl3d").unwrap();
    let model_id = rendering::pipeline::pipec::construct(&pipeline, model).unwrap();

    // Create it's material
    let texture = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rock_diffuse.png")
        .unwrap()
        .set_mipmaps(true);
    let texture = rendering::pipeline::pipec::construct(&pipeline, texture).unwrap();

    let texture2 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rock_normal.png")
        .unwrap()
        .set_mipmaps(true);
    let texture2 = rendering::pipeline::pipec::construct(&pipeline, texture2).unwrap();

    let material = rendering::basics::material::Material::default()
        .with_diffuse(texture)
        .with_normal(texture2)
        .with_normal_strength(0.3)
        .with_uv_scale(veclib::Vector2::ONE * 3.0);
    let material = rendering::pipeline::pipec::construct(&pipeline, material).unwrap();

    // Create a simple cube
    for x in 0..1 {
        for y in 0..1 {
            let mut group = ecs::entity::ComponentLinkingGroup::default();
            let entity = ecs::entity::Entity::default();
            let id = ecs::entity::EntityID::new(&mut world.ecs);
            let transform = defaults::components::Transform::default()
                .with_position(veclib::vec3(0.0, 10.0, 0.0))
                .with_scale(veclib::vec3(50.2, 2.0, 5.0));
            let matrix = transform.calculate_matrix();
            group.link::<defaults::components::Transform>(transform).unwrap();
            group.link_default::<defaults::components::Physics>().unwrap();

            // Create it's renderer
            let renderer = rendering::basics::renderer::Renderer::new(rendering::basics::renderer::RendererFlags::DEFAULT)
                .with_model(model_id)
                .with_material(material)
                .with_matrix(matrix);
            let renderer = defaults::components::Renderer::new(renderer);
            group.link(renderer).unwrap();
            // Add the cube
            world.ecs.add_entity(entity, id, group).unwrap();
        }
    }

    // Create the directional light source
    let light = rendering::basics::lights::LightSource::new(rendering::basics::lights::LightSourceType::Directional {
        quat: veclib::Quaternion::IDENTITY,
    })
    .with_strength(1.0);
    let mut world_global = world.globals.get_global_mut::<defaults::globals::GlobalWorldData>().unwrap();
    world_global.sun_quat = veclib::Quaternion::<f32>::from_axis_angle(veclib::Vector3::X, 80.0);
    rendering::pipeline::pipec::construct(&pipeline, light).unwrap();
    // Load a terrain material
    // Load the shader first
    let settings = rendering::basics::shader::ShaderSettings::default()
        .source("defaults\\shaders\\voxel_terrain\\terrain.vrsh.glsl")
        .source("defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    let shader = rendering::pipeline::pipec::construct(&pipeline, rendering::basics::shader::Shader::new(settings).unwrap()).unwrap();
    // Then the textures
    let _white = pipeline.textures.get(pipeline.defaults.as_ref().unwrap().white).unwrap();
    let _normal_map = pipeline.textures.get(pipeline.defaults.as_ref().unwrap().normals_tex).unwrap();
    let texture_diff_1 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\forrest_ground_01_diff_2k.jpg").unwrap();
    let texture_norm_1 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\forrest_ground_01_nor_gl_2k.jpg").unwrap();
    let texture_diff_2 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_3 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rocks_ground_08_diff_2k.jpg").unwrap();
    let texture_norm_3 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rocks_ground_08_nor_gl_2k.jpg").unwrap();
    let diffuse = rendering::basics::texture::Texture::convert_texturearray(vec![&texture_diff_1, &texture_diff_2, &texture_diff_3])
        .unwrap()
        .set_mipmaps(true)
        .set_filter(rendering::basics::texture::TextureFilter::Linear);
    let normals = rendering::basics::texture::Texture::convert_texturearray(vec![&texture_norm_1, &texture_norm_2, &texture_norm_3])
        .unwrap()
        .set_mipmaps(true)
        .set_filter(rendering::basics::texture::TextureFilter::Linear);

    let diffuse = rendering::pipeline::pipec::construct(&pipeline, diffuse).unwrap();
    let normals = rendering::pipeline::pipec::construct(&pipeline, normals).unwrap();
    let material = rendering::basics::material::Material::default()
        .with_diffuse(diffuse)
        .with_normal(normals)
        .with_normal_strength(0.0)
        .with_uv_scale(veclib::Vector2::ONE * 0.02)
        .with_shader(shader);
    let material = rendering::pipeline::pipec::construct(&pipeline, material).unwrap();
    let heuristic = math::octrees::HeuristicSettings::new(|node, target| {
        let dist = veclib::Vector3::<f32>::distance(node.get_center().into(), *target) / (node.half_extent as f32 * 2.0);
        dist < 1.2
    });
    let tex = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\saber.png").unwrap();
    let _tex = rendering::pipeline::pipec::construct(&pipeline, tex).unwrap();
    // Add the terrain
    drop(pipeline);
    let terrain = defaults::globals::Terrain::new("user\\shaders\\voxel_terrain\\voxel.func.glsl", 4, &pipeline_)
        .set_heuristic(heuristic)
        .set_material(material);
        //.set_material(material);
    world.globals.add_global(terrain).unwrap();
}
