use cflake_engine::{
    assets::assetc,
    defaults::{
        components::{self, Light, Transform},
        globals::{self, TerrainSettings},
    },
    ecs::entity::{ComponentLinkingGroup, Entity},
    math::{
        csg::CSGOperation,
        octrees::HeuristicSettings,
        shapes::{Cuboid, ShapeType, Sphere},
    },
    rendering::basics::{
        lights::{LightParameters, LightType::Directional},
        material::{Material, MaterialTextures},
        shader::{Shader, ShaderInitSettings},
        texture::{Texture, TextureBuilder, TextureBundler, TextureFilter, TextureLayout},
    },
    terrain::editing::Edit,
    veclib::{self, vec3},
    World,
};

// A game with some test terrain
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-terrain", init)
}
// Init the terrain world

fn init(world: &mut World) {
    cflake_engine::assets::init!("/examples/assets/");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/voxel.func.glsl");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/voxel.func.glsl");
    cflake_engine::assets::asset!("./assets/user/textures/forrest_ground_01_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/forrest_ground_01_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_diff_8k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_nor_gl_8k.jpg");
    // Load le assets

    // ----Start the world----
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(components::Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link_default::<components::Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.add(entity, group).unwrap();

    // Create the directional light source
    let light = components::Light {
        light: Directional {
            params: LightParameters::default(),
        },
    };
    let light_transform = components::Transform::default().with_rotation(veclib::Quaternion::<f32>::from_x_angle(-90f32.to_radians()));
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(Entity::default(), group).unwrap();

    // Load a terrain material
    // Load the shader first
    let settings = ShaderInitSettings::default()
        .source("defaults/shaders/voxel_terrain/terrain.vrsh.glsl")
        .source("defaults/shaders/voxel_terrain/terrain.frsh.glsl");
    let shader = world.pipeline.shaders.insert(Shader::new(settings).unwrap());
    // Then the textures
    let texture_diff_1 = assetc::load::<Texture>("user/textures/forrest_ground_01_diff_2k.jpg").unwrap();
    let texture_norm_1 = assetc::load::<Texture>("user/textures/forrest_ground_01_nor_gl_2k.jpg").unwrap();
    let texture_diff_2 = assetc::load::<Texture>("user/textures/rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 = assetc::load::<Texture>("user/textures/rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_3 = assetc::load::<Texture>("user/textures/rocks_ground_08_diff_2k.jpg").unwrap();
    let texture_norm_3 = assetc::load::<Texture>("user/textures/rocks_ground_08_nor_gl_2k.jpg").unwrap();
    let diffuse = TextureBundler::convert_texturearray(&[texture_diff_1, texture_diff_2, texture_diff_3]).mipmaps(true);
    let normals = TextureBundler::convert_texturearray(&[texture_norm_1, texture_norm_2, texture_norm_3]).mipmaps(true);
    let diffuse = world.pipeline.textures.insert(diffuse.build());
    let normals = world.pipeline.textures.insert(normals.build());
    let material = Material {
        shader,
        textures: MaterialTextures {
            diffuse_map: diffuse,
            normal_map: normals,
            ..Default::default()
        },
        uv_scale: veclib::Vector2::ONE * 0.03,
        ..Default::default()
    };
    let material = world.pipeline.materials.insert(material);
    let heuristic = HeuristicSettings::default()
        .with_function(|node, target| {
            let dist = veclib::Vector3::<f32>::distance(node.center().into(), *target) / (node.half_extent() as f32 * 2.0);
            dist < 1.2
        })
        .with_threshold(64.0);
    // Create some terrain settings
    let terrain_settings = TerrainSettings {
        voxel_src_path: "user/shaders/voxel_terrain/voxel.func.glsl".to_string(),
        depth: 5,
        heuristic_settings: heuristic,
        material,
        ..Default::default()
    };
    let mut terrain = globals::Terrain::new(terrain_settings, &mut world.pipeline);
    // Big sphere
    terrain.edit(Edit::sphere(veclib::Vector3::ZERO, 500.0, CSGOperation::Union, Some(1)));
    // Pillar
    terrain.edit(Edit::cuboid(veclib::Vector3::ZERO, vec3(400.0, 600.0, 400.0), CSGOperation::Subtraction, Some(2)));
    world.globals.add(terrain).unwrap();
}
