use cflake_engine::*;
use window::start;
fn main() {
    // Load up the engine
    start("DevJed", "DevGame", preload_assets, init);
}
pub fn preload_assets() {
    // -----Pre-load the game assets here-----
}
pub fn init(mut write: window::core::WriteContext) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ecs::entity::ComponentLinkingGroup::new();
    group.link(defaults::components::Camera::new(90.0, 0.5, 1000.0)).unwrap();
    group.link_default::<defaults::components::Transform>().unwrap();
    let entity = ecs::entity::Entity::new();
    let id = ecs::entity::EntityID::new(&mut write.ecs);
    write.ecs.add_entity(entity, id, group).unwrap();



    // Create it's model
    let (model_id, material, texture, texture2) = {
        let pipeline = write.pipeline.read().unwrap();
        let model = assets::assetc::dload::<rendering::basics::model::Model>("defaults\\models\\cube.mdl3d").unwrap();
        let model_id = rendering::pipeline::pipec::construct(model, &*pipeline);

        // Create it's material
        let texture = assets::assetc::dload::<rendering::basics::texture::Texture>("defaults\\textures\\rock_diffuse.png").unwrap();
        let texture = rendering::pipeline::pipec::construct(texture, &*pipeline);

        let texture2 = assets::assetc::dload::<rendering::basics::texture::Texture>("defaults\\textures\\rock_normal.png").unwrap();
        let texture2 = rendering::pipeline::pipec::construct(texture2, &*pipeline);

        let material = rendering::basics::material::Material::default().set_diffuse_texture(texture).set_normals_texture(texture2);
        let material = rendering::pipeline::pipec::construct(material, &*pipeline);
        (model_id, material, texture, texture2)
    };

    // Create a simple cube
    for x in 0..2 {
        for y in 0..2 {
            let mut group = ecs::entity::ComponentLinkingGroup::new();
            let entity = ecs::entity::Entity::new();
            let id = ecs::entity::EntityID::new(&mut write.ecs);
            let matrix = defaults::components::Transform::default().calculate_matrix();
            group.link::<defaults::components::Transform>(defaults::components::Transform::default().with_position(veclib::vec3(y as f32 * 2.2, 0.0, x as f32 * 2.2))).unwrap();
            group.link_default::<defaults::components::Physics>().unwrap();      
            
            // Create it's renderer
            let renderer = rendering::basics::renderer::Renderer::default().set_model(model_id).set_material(material).set_matrix(matrix);
            let renderer = defaults::components::Renderer::new(renderer);
            group.link(renderer).unwrap();
            // Add the cube
            write.ecs.add_entity(entity, id, group);
        }
    }
}
