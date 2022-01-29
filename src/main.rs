use cflake_engine::*;
fn main() {
    // Load up the engine
    start("DevJed", "DevGame", preload_assets, init);
}
pub fn preload_assets() {
    // -----Pre-load the game assets here-----
    assets::preload_asset!(".\\resources\\user\\textures\\rock_diffuse.png");
    assets::preload_asset!(".\\resources\\user\\textures\\rock_normal.png");
}
pub fn init(mut write: core::WriteContext) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ecs::entity::ComponentLinkingGroup::new();
    group.link(defaults::components::Camera::new(90.0, 0.5, 1000.0)).unwrap();
    group.link_default::<defaults::components::Transform>().unwrap();
    let entity = ecs::entity::Entity::new();
    let id = ecs::entity::EntityID::new(&mut write.ecs);
    write.ecs.add_entity(entity, id, group).unwrap();
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read().unwrap();
    // Create it's model
    let mut model = assets::assetc::dload::<rendering::basics::model::Model>("defaults\\models\\sphere.mdl3d").unwrap();
    let model_id = rendering::pipeline::pipec::construct(model, &*pipeline);

    // Create it's material
    let texture = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rock_diffuse.png")
        .unwrap()
        .set_mipmaps(true);
    let texture = rendering::pipeline::pipec::construct(texture, &*pipeline);

    let texture2 = assets::assetc::dload::<rendering::basics::texture::Texture>("user\\textures\\rock_normal.png")
        .unwrap()
        .set_mipmaps(true);
    let texture2 = rendering::pipeline::pipec::construct(texture2, &*pipeline);

    let material = rendering::basics::material::Material::default()
        .set_diffuse_texture(texture)
        .set_normals_texture(texture2)
        .set_normals_strength(0.3)
        .set_uv_scale(veclib::Vector2::ONE * 3.0);
    let material = rendering::pipeline::pipec::construct(material, &*pipeline);

    // Create a simple cube
    for x in 0..2 {
        for y in 0..2 {
            let mut group = ecs::entity::ComponentLinkingGroup::new();
            let entity = ecs::entity::Entity::new();
            let id = ecs::entity::EntityID::new(&mut write.ecs);
            let matrix = defaults::components::Transform::default().calculate_matrix();
            group
                .link::<defaults::components::Transform>(defaults::components::Transform::default().with_position(veclib::vec3(y as f32 * 2.2, 0.0, x as f32 * 2.2)))
                .unwrap();
            group.link_default::<defaults::components::Physics>().unwrap();

            // Create it's renderer
            let renderer = rendering::basics::renderer::Renderer::default()
                .set_model(model_id)
                .set_material(material)
                .set_matrix(matrix);
            let renderer = defaults::components::Renderer::new(renderer);
            group.link(renderer).unwrap();
            // Add the cube
            write.ecs.add_entity(entity, id, group).unwrap();
        }
    }

    // Load a terrain material
    // Load the shader first
    let ss = rendering::basics::shader::ShaderSettings::default()
        .source("defaults\\shaders\\rendering\\default.vrsh.glsl")
        .source("defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    let shader = rendering::pipeline::pipec::construct(rendering::basics::shader::Shader::new(ss).unwrap(), &*pipeline);
    // Then the textures
    let white = pipeline.get_texture(pipeline.defaults.as_ref().unwrap().white).unwrap();
    let normal_map = pipeline.get_texture(pipeline.defaults.as_ref().unwrap().normals_tex).unwrap();
    let diffuse = rendering::basics::texture::Texture::convert_3d(vec![white]).unwrap();
    let normals = rendering::basics::texture::Texture::convert_3d(vec![normal_map]).unwrap();

    let diffuse = rendering::pipeline::pipec::construct(diffuse, &*pipeline);
    let normals = rendering::pipeline::pipec::construct(normals, &*pipeline);

    let material = rendering::basics::material::Material::default()
        .set_diffuse_texture(diffuse)
        .set_normals_texture(normals)
        .set_shader(shader);
    let material = rendering::pipeline::pipec::construct(material, &*pipeline);

    // Add the terrain
    //let terrain = defaults::globals::Terrain::new(material, 6, &*pipeline);
    //write.ecs.add_global(terrain).unwrap();
}
