use cflake_engine::{
    assets::assetc,
    defaults::{
        components,
        globals::{self, TerrainSettings},
    },
    ecs::entity::{ComponentLinkingGroup, Entity},
    math::{
        csg::CSGOperation,
        octrees::HeuristicSettings,
        shapes::{BasicShapeType, Cuboid, Sphere},
    },
    rendering::{
        basics::{
            lights::{LightSource, LightSourceType},
            material::Material,
            shader::{Shader, ShaderSettings},
            texture::{Texture, TextureFilter},
        },
        pipeline::pipec,
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
    group
        .link(components::Camera::new(90.0, 2.0, 9000.0))
        .unwrap();
    group.link_default::<components::Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.add_entity(entity, group).unwrap();
    let pipeline = world.pipeline.read();
    // Create the directional light source
    let light = LightSource::new(LightSourceType::Directional {
        quat: veclib::Quaternion::<f32>::from_x_angle(-45f32.to_radians()),
    })
    .with_strength(1.3);
    pipec::construct(&pipeline, light).unwrap();
    // Load a terrain material
    // Load the shader first
    let settings = ShaderSettings::default()
        .source("defaults/shaders/voxel_terrain/terrain.vrsh.glsl")
        .source("defaults/shaders/voxel_terrain/terrain.frsh.glsl");
    let shader = pipec::construct(&pipeline, Shader::new(settings).unwrap()).unwrap();
    // Then the textures
    let _white = pipeline
        .textures
        .get(pipeline.defaults.as_ref().unwrap().white)
        .unwrap();
    let _normal_map = pipeline
        .textures
        .get(pipeline.defaults.as_ref().unwrap().normals_tex)
        .unwrap();
    let texture_diff_1 =
        assetc::load::<Texture>("user/textures/forrest_ground_01_diff_2k.jpg").unwrap();
    let texture_norm_1 =
        assetc::load::<Texture>("user/textures/forrest_ground_01_nor_gl_2k.jpg").unwrap();
    let texture_diff_2 =
        assetc::load::<Texture>("user/textures/rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 =
        assetc::load::<Texture>("user/textures/rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_3 =
        assetc::load::<Texture>("user/textures/rocks_ground_08_diff_2k.jpg").unwrap();
    let texture_norm_3 =
        assetc::load::<Texture>("user/textures/rocks_ground_08_nor_gl_2k.jpg").unwrap();
    let diffuse =
        Texture::convert_texturearray(vec![&texture_diff_1, &texture_diff_2, &texture_diff_3])
            .unwrap()
            .with_mipmaps(true)
            .with_filter(TextureFilter::Linear);
    let normals =
        Texture::convert_texturearray(vec![&texture_norm_1, &texture_norm_2, &texture_norm_3])
            .unwrap()
            .with_mipmaps(true)
            .with_filter(TextureFilter::Linear);

    let diffuse = pipec::construct(&pipeline, diffuse).unwrap();
    let normals = pipec::construct(&pipeline, normals).unwrap();
    let material = Material::default()
        .with_diffuse(diffuse)
        .with_normal(normals)
        .with_normal_strength(0.7)
        .with_uv_scale(veclib::Vector2::ONE * 0.03)
        .with_shader(shader);
    let material = pipec::construct(&pipeline, material).unwrap();
    let heuristic = HeuristicSettings::default()
        .with_function(|node, target| {
            let dist = veclib::Vector3::<f32>::distance(node.center().into(), *target)
                / (node.half_extent as f32 * 2.0);
            dist < 1.2
        })
        .with_threshold(64.0);
    // Create some terrain settings
    let terrain_settings = TerrainSettings::default()
        .with_depth(5)
        .with_material(material)
        .with_heuristic(heuristic)
        .with_voxel_src("user/shaders/voxel_terrain/voxel.func.glsl");
    let mut terrain = globals::Terrain::new(terrain_settings, &pipeline);
    terrain.edit(
        Edit::new(
            BasicShapeType::Sphere(Sphere {
                center: veclib::Vector3::ZERO,
                radius: 500.0,
            }),
            CSGOperation::Subtraction,
        )
        .with_material(2),
    );
    terrain.edit(
        Edit::new(
            BasicShapeType::Cuboid(Cuboid {
                center: veclib::Vector3::ZERO,
                size: vec3(200.0, 6000.0, 200.0),
            }),
            CSGOperation::Union,
        )
        .with_material(2),
    );
    world.globals.add_global(terrain).unwrap();
}